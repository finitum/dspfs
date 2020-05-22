use crate::message::{Message, SignedMessage};
use crate::user::{PrivateUser, PublicUser};
use anyhow::{Context, Result};
use async_trait::async_trait;
use ring::aead::*;
use ring::agreement;
use ring::agreement::{EphemeralPrivateKey, PublicKey, UnparsedPublicKey};
use ring::error::Unspecified;
use ring::pbkdf2::{derive, PBKDF2_HMAC_SHA256};
use serde::export::Formatter;
use std::fmt;
use std::fmt::Debug;
use std::num::NonZeroU32;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[async_trait]
trait WriteWithLength {
    async fn write_with_length(&mut self, bytes: &[u8]) -> Result<()>;
}

#[async_trait]
impl<T: AsyncWriteExt + Unpin + Send + Sync> WriteWithLength for T {
    async fn write_with_length(&mut self, bytes: &[u8]) -> Result<()> {
        self.write_u64(bytes.len() as u64).await?;
        self.write_all(bytes).await?;
        Ok(())
    }
}

#[async_trait]
trait ReadWithLength {
    /// Reads a message with a length specified in the message.
    /// This length is blindly trusted, and therefore is quite dangerous.
    async fn read_with_length(&mut self) -> Result<Vec<u8>> {
        self.read_with_length_limited(0)
            .await
            .context("reading failed")
    }

    /// Reads a message with a length specified in the message.
    /// Aborts reading when the message length is larger than the limit.
    /// A limit of 0 means no limit. This function is much safer than read_with_length.
    async fn read_with_length_limited(&mut self, limit: usize) -> Result<Vec<u8>>;
}

#[async_trait]
impl<T: AsyncReadExt + Unpin + Send + Sync> ReadWithLength for T {
    async fn read_with_length_limited(&mut self, limit: usize) -> Result<Vec<u8>> {
        let mut res = Vec::new();

        let mut length = self
            .read_u64()
            .await
            .context("Failed to read message length")?;

        if length as usize > limit && limit != 0 {
            return Err(anyhow::anyhow!("Message length was larger than read limit"));
        }

        while length > 0 {
            let size = length.min(1024);
            length -= size;
            let mut buf = vec![0u8; size as usize];

            self.read_exact(&mut buf)
                .await
                .context("Failed to read stream")?;
            res.extend_from_slice(&buf)
        }

        Ok(res)
    }
}

/// EncryptedStream is a wrapper around any stream-like object to setup an
/// end to end crypted tunnel.
/// It first uses ECDH to make a shared secret and to verify identity
/// then it uses said shared secret to derive a CHACHA20_POLY1305 keypair
/// for use for further communication.
pub struct EncryptedStream<T: AsyncReadExt + AsyncWriteExt + Unpin> {
    stream: T,
    other_user: PublicUser,

    // symmetric key pair
    opening_key: OpeningKey<NonceGenerator>,
    sealing_key: SealingKey<NonceGenerator>,
}

impl<T: AsyncReadExt + AsyncWriteExt + Unpin + Send + Sync> Debug for EncryptedStream<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "EncryptedStream{{user: {:?}}}", self.other_user)
    }
}

type KDF = dyn FnOnce(&[u8]) -> std::result::Result<[u8; 32], ring::error::Unspecified>;

// TODO: Verify PublicKey
impl<T: AsyncReadExt + AsyncWriteExt + Unpin + Send + Sync> EncryptedStream<T> {
    /// Generates a ephemeral keypair for use with ECDH using Ring
    fn generate_ephemeral_keypair() -> Result<(PublicKey, EphemeralPrivateKey)> {
        // FIXME: should be passed down
        let rng = ring::rand::SystemRandom::new();

        // Generate our ephemeral keypair
        let my_private_key = EphemeralPrivateKey::generate(&agreement::X25519, &rng)
            .map_err(|_| anyhow::anyhow!("unspecified ring error"))?;
        let my_public_key = my_private_key
            .compute_public_key()
            .map_err(|_| anyhow::anyhow!("unspecified ring error"))?;

        Ok((my_public_key, my_private_key))
    }

    /// Creates a symmetric CHACHA20_POLY1305 keypair to encrypt and decrypt all communiction
    /// the shared_key should have been obtained using ECDH
    fn create_symmetric_keypair(
        shared_key: &[u8],
    ) -> Result<(OpeningKey<NonceGenerator>, SealingKey<NonceGenerator>)> {
        // TODO: Is this good practice? Should you make a sealing key and opening key from one shared secret?
        //       Should we use different salts for the sealer and opener? Maybe use our username as sealer and
        //       the other user's name as opener?
        let unbound_key1 = UnboundKey::new(&CHACHA20_POLY1305, &shared_key)
            .map_err(|_| anyhow::anyhow!("unspecified ring error"))?;
        let unbound_key2 = UnboundKey::new(&CHACHA20_POLY1305, &shared_key)
            .map_err(|_| anyhow::anyhow!("unspecified ring error"))?;

        let opening_key = OpeningKey::new(unbound_key1, NonceGenerator::default());
        let sealing_key = SealingKey::new(unbound_key2, NonceGenerator::default());

        Ok((opening_key, sealing_key))
    }

    /// initiator is used to initiate an EncryptedStream
    /// this will use ECDH for key exchange and CHACHA20_POLY1305 as symmetric encryption
    pub async fn initiator(mut stream: T, user: &PrivateUser) -> Result<Self> {
        let (my_public_key, my_private_key) = Self::generate_ephemeral_keypair()?;

        let my_init_msg = Message::Init {
            user: user.public_user().to_owned(),
            pubkey: my_public_key.as_ref().to_vec(),
        };

        // Sign the init message
        let signedmsg = my_init_msg
            .sign(user.get_keypair())
            .context("failed to sign message")?
            .serialize()
            .context("failed to serialize message")?;

        // Send a message to the user we want to connect to to initiate the Diffie Helmann exchange
        stream
            .write_with_length(&signedmsg)
            .await
            .context("failed to write message to stream")?;

        // Deserialize and verify the message we get back
        let other_init_message = Self::read_signed_message(&mut stream)
            .await
            .context("failed to read response from stream")?;
        let (other_user, peer_public_key) = Self::extract_verify(&other_init_message)
            .context("could not verify the identity of the incoming message")?;

        // PBKDF
        let shared_key: [u8; 32] = ring::agreement::agree_ephemeral(
            my_private_key,
            &peer_public_key,
            ring::error::Unspecified,
            Self::kdff(user.public_user().to_owned(), other_user.clone()),
        )
        .map_err(|_| anyhow::anyhow!("unspecified ring error"))?;

        let (opening_key, sealing_key) = Self::create_symmetric_keypair(&shared_key)
            .context("failed to generate symmetric keypair from shared secret")?;

        Ok(Self {
            stream,
            other_user,
            opening_key,
            sealing_key,
        })
    }

    /// receiver is used to receive an EncryptedStream
    /// this will use ECDH for key exchange and CHACHA20_POLY1305 as symmetric encryption
    pub async fn receiver(mut stream: T, user: PrivateUser) -> Result<Self> {
        // Generate our ephemeral keypair
        let (my_public_key, my_private_key) =
            Self::generate_ephemeral_keypair().context("failed to generate ephemeral keypair")?;

        let signed_init_message = Self::read_signed_message(&mut stream)
            .await
            .context("failed to read message from stream")?;

        // Extract message parts and apply diffie hellman to create a shared secret
        let (other_user, peer_public_key) = Self::extract_verify(&signed_init_message)
            .context("could not verify the identity of the incoming message")?;

        // Now we know the identity of the other user, send something back
        let our_init_msg = Message::Init {
            user: user.public_user().to_owned(),
            pubkey: my_public_key.as_ref().to_vec(),
        };

        // Sign, Seal, Deliver
        let signedmsg = our_init_msg
            .sign(user.get_keypair())
            .context("failed to sign message")?
            .serialize()
            .context("failed to serialize message")?;
        stream
            .write_with_length(&signedmsg)
            .await
            .context("failed to write message to stream")?;

        // TODO: Is this good practice? Should you make a sealing key and opening key from one shared secret?
        //       Should we use different salts for the sealer and opener? Maybe use our username as sealer and
        //       the other user's name as opener?
        let shared_key = ring::agreement::agree_ephemeral(
            my_private_key,
            &peer_public_key,
            ring::error::Unspecified,
            Self::kdff(other_user.clone(), user.public_user().to_owned()),
        )
        .map_err(|_| anyhow::anyhow!("unspecified ring error"))?;

        // Generate the CHACHA20_POLY1305 keypair
        let (opening_key, sealing_key) = Self::create_symmetric_keypair(&shared_key)
            .context("failed to generate symmetric keypair from shared secret")?;

        Ok(Self {
            stream,
            other_user,
            opening_key,
            sealing_key,
        })
    }

    /// kdff returns a key derivation function to be used with ring's derive.
    /// it uses PBKDF2_HMAC_SHA256 as the algorithm for this
    /// and (for now) concats the intitiator's username and receiver's username as salt.
    fn kdff(initiator: PublicUser, receiver: PublicUser) -> Box<KDF> {
        Box::new(move |key_material| {
            let mut salt = initiator.get_username().to_owned();
            salt.push_str(receiver.get_username());

            let mut shared_key = [0; 32];
            derive(
                PBKDF2_HMAC_SHA256,
                // DO NOT MAKE 0
                NonZeroU32::new(42u32).unwrap(),
                salt.as_bytes(),
                key_material,
                &mut shared_key,
            );

            Ok(shared_key)
        })
    }

    /// extract_verify extracts and verifies theessage using the embedded key
    /// you should verify that the PublicUser matches thene you expect.
    fn extract_verify(
        signed_message: &SignedMessage,
    ) -> Result<(PublicUser, UnparsedPublicKey<Vec<u8>>)> {
        let message: Message = bincode::deserialize(&signed_message.message)
            .context("failed to deserialize message")?;

        // Extract message
        let (user, key) = match message {
            Message::Init { user, pubkey } => (
                user,
                agreement::UnparsedPublicKey::new(&agreement::X25519, pubkey),
            ),
            _ => {
                return Err(anyhow::anyhow!(
                    "unexpected message type found: expected connection initialization message"
                ))
            }
        };

        // Check signature when we know their public key
        // TODO: Maybe take another user as argso we can verify identity early
        user.get_public_key()
            .ring()
            .verify(&signed_message.message, &signed_message.signature)
            .map_err(|_| anyhow::anyhow!("unspecified ring error"))?;
        Ok((user, key))
    }

    /// Reads one signed message and decodes it. Just to make other functions in this struct a little smaller and more readable.
    async fn read_signed_message(stream: &mut T) -> Result<SignedMessage> {
        let message = stream
            .read_with_length_limited(1024)
            .await
            .context("failed to read message")?;
        Ok(bincode::deserialize(&message)?)
    }

    /// Encrypts + Sends a [Message]
    pub async fn send_message(&mut self, message: Message) -> Result<()> {
        let mut serialized_message =
            bincode::serialize(&message).context("failed to serialize message")?;
        self.sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut serialized_message)
            .map_err(|_| anyhow::anyhow!("unspecified ring error"))?;

        self.stream
            .write_with_length(&serialized_message)
            .await
            .context("failed to write message")?;

        Ok(())
    }

    /// Decrypts + Receives a [Message]
    pub async fn recv_message(&mut self, limit: usize) -> Result<Message> {
        let mut msg = self
            .stream
            .read_with_length_limited(limit)
            .await
            .context("failed to read message")?;

        self.opening_key
            .open_in_place(Aad::empty(), &mut msg)
            .map_err(|_| anyhow::anyhow!("unspecified ring error"))?;

        let dmsg = bincode::deserialize(&msg).context("failed to deserialize message")?;

        Ok(dmsg)
    }
}

struct NonceGenerator {
    value: u128,
}

impl Default for NonceGenerator {
    fn default() -> Self {
        NonceGenerator { value: 0 }
    }
}

impl NonceSequence for NonceGenerator {
    fn advance(&mut self) -> std::result::Result<Nonce, Unspecified> {
        self.value += 1;

        let mut bytes = [0u8; NONCE_LEN];

        bytes.copy_from_slice(&self.value.to_le_bytes()[..NONCE_LEN]);

        Ok(Nonce::assume_unique_for_key(bytes))
    }
}

#[cfg(test)]
mod tests {
    use crate::init;
    use crate::message::Message;
    use crate::stream::encryptedstream::{EncryptedStream, NonceGenerator};
    use crate::user::PrivateUser;
    use log::*;
    use ring::aead::NonceSequence;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::{delay_for, Duration};

    #[test]
    fn test_not_0_nonce() {
        let n = NonceGenerator::default().advance().unwrap();

        assert_ne!(*n.as_ref(), [0u8; 12])
    }

    #[test]
    fn test_nonce_uniqueness() {
        let mut n = NonceGenerator::default();
        let a = n.advance().unwrap();
        let b = n.advance().unwrap();

        assert_ne!(a.as_ref(), b.as_ref())
    }

    #[tokio::test]
    async fn test_encrypted_stream() {
        init();

        let (u1, _) = PrivateUser::new("test1").unwrap();
        let (u2, _) = PrivateUser::new("test2").unwrap();

        const MSG: &str = "asd";

        tokio::spawn(async move {
            info!("Start listening");

            let mut listener = TcpListener::bind("localhost:8984").await.unwrap();
            let (stream, _) = listener.accept().await.unwrap();

            info!("Got connection");

            let mut er = EncryptedStream::receiver(stream, u2).await.unwrap();

            let mut rmsg = er.recv_message(0).await.unwrap();
            match rmsg {
                Message::String(s) => assert_eq!(s, MSG),
                _ => unreachable!(),
            }

            rmsg = er.recv_message(0).await.unwrap();

            match rmsg {
                Message::String(s) => assert_eq!(s, MSG),
                _ => unreachable!(),
            }

            rmsg = er.recv_message(0).await.unwrap();

            match rmsg {
                Message::String(s) => assert_eq!(s, MSG),
                _ => unreachable!(),
            }
        });

        delay_for(Duration::from_secs_f64(0.5)).await;

        info!("Sending");

        let sock = TcpStream::connect("localhost:8984").await.unwrap();
        let mut es = EncryptedStream::initiator(sock, &u1).await.unwrap();

        es.send_message(Message::String(MSG.into())).await.unwrap();
        es.send_message(Message::String(MSG.into())).await.unwrap();
        es.send_message(Message::String(MSG.into())).await.unwrap();

        delay_for(Duration::from_secs_f64(0.5)).await;

        // dbg!(es);
    }
}
