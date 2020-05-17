use crate::error::DspfsError;
use crate::message::{Message, SignedMessage};
use crate::user::{PrivateUser, PublicUser};
use async_trait::async_trait;
use ring::aead::{
    Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, CHACHA20_POLY1305,
    NONCE_LEN,
};
use ring::error::Unspecified;
use ring::pbkdf2::*;
use serde::export::Formatter;
use std::fmt;
use std::fmt::Debug;
use std::num::NonZeroU32;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use x25519_dalek::EphemeralSecret;
use x25519_dalek::PublicKey;

#[async_trait]
trait WriteWithLength {
    async fn write_with_length(&mut self, bytes: &[u8]) -> io::Result<()>;
}

#[async_trait]
impl<T: AsyncWriteExt + Unpin + Send + Sync> WriteWithLength for T {
    async fn write_with_length(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.write_u64(bytes.len() as u64).await?;
        self.write_all(bytes).await?;
        Ok(())
    }
}

#[async_trait]
trait ReadWithLength {
    /// Reads a message with a length specified in the message.
    /// This length is blindly trusted, and therefore is quite dangerous.
    async fn read_with_length(&mut self) -> io::Result<Vec<u8>> {
        self.read_with_length_limited(0).await
    }

    /// Reads a message with a length specified in the message.
    /// Aborts reading when the message length is larger than the limit.
    /// A limit of 0 means no limit. This function is much safer than read_with_length.
    async fn read_with_length_limited(&mut self, limit: usize) -> io::Result<Vec<u8>>;
}

#[async_trait]
impl<T: AsyncReadExt + Unpin + Send + Sync> ReadWithLength for T {
    async fn read_with_length_limited(&mut self, limit: usize) -> io::Result<Vec<u8>> {
        let mut res = Vec::new();

        let mut length = self.read_u64().await?;

        if length as usize > limit && limit != 0 {
            return Err(io::ErrorKind::Interrupted.into());
        }

        while length > 0 {
            let size = length.min(1024);
            length -= size;
            let mut buf = vec![0u8; size as usize];

            self.read_exact(&mut buf).await?;
            res.extend_from_slice(&buf)
        }

        Ok(res)
    }
}

pub struct EncryptedStream<T: AsyncReadExt + AsyncWriteExt + Unpin> {
    stream: T,
    other_user: PublicUser,

    pub(self) opening_key: OpeningKey<NonceGenerator>,
    pub(self) sealing_key: SealingKey<NonceGenerator>,
}

impl<T: AsyncReadExt + AsyncWriteExt + Unpin + Send + Sync> Debug for EncryptedStream<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "EncryptedStream{{user: {:?}}}", self.other_user)
    }
}

impl<T: AsyncReadExt + AsyncWriteExt + Unpin + Send + Sync> EncryptedStream<T> {
    pub async fn initiator(mut stream: T, user: &PrivateUser) -> Result<Self, DspfsError> {
        // TODO: Maybe switch to ring for DH
        // TODO: Refactor to be independent from TcpStream

        let our_partial_secret = EphemeralSecret::new(&mut rand_core::OsRng);
        let our_pubkey = PublicKey::from(&our_partial_secret);

        let our_init_msg = Message::Init {
            user: user.public_user().to_owned(),
            pubkey: our_pubkey,
        };

        let signedmsg = our_init_msg.sign(user.get_keypair())?.serialize()?;

        // Send a message to the user we want to connect to to initiate the Diffie Helmann exchange
        stream.write_with_length(&signedmsg).await?;

        // Deserialize the message we get back
        let other_init_message = Self::read_signed_message(&mut stream).await?;
        let other_user_signed_message: Message = bincode::deserialize(&other_init_message.message)?;

        // Extract message parts and apply diffie hellman to create a shared secret
        let (shared_secret, other_user) = match other_user_signed_message {
            Message::Init {
                user,
                pubkey: other_pubkey,
            } => (our_partial_secret.diffie_hellman(&other_pubkey), user),
            _ => return Err(DspfsError::InvalidEncryptedConnectionInitialization),
        };

        // Check signature when we know their public key
        other_user
            .get_public_key()
            .ring()
            .verify(&other_init_message.message, &other_init_message.signature)
            .map_err(|_| DspfsError::BadSignature)?;

        // TODO: Less hardcoding of thingies

        // Derive our actual symmetric keys so we can have encrypted communication
        // WATCH OUT: REVERSE USERNAMES ON RECEIVING SIDE TO MAKE EQUAL SALTS
        let mut salt = user.get_username().clone();
        salt.push_str(other_user.get_username());
        let mut shared_key = [0; 32];
        // TODO: Derive may panic, maybe check for this?
        derive(
            PBKDF2_HMAC_SHA256,
            // DO NOT MAKE 0
            NonZeroU32::new(42u32).unwrap(),
            salt.as_bytes(),
            shared_secret.as_bytes(),
            &mut shared_key,
        );

        // TODO: Is this good practice? Should you make a sealing key and opening key from one shared secret?
        //       Should we use different salts for the sealer and opener? Maybe use our username as sealer and
        //       the other user's name as opener?
        let unbound_key1 = UnboundKey::new(&CHACHA20_POLY1305, &shared_key)?;
        let unbound_key2 = UnboundKey::new(&CHACHA20_POLY1305, &shared_key)?;

        let opening_key = OpeningKey::new(unbound_key1, NonceGenerator::default());
        let sealing_key = SealingKey::new(unbound_key2, NonceGenerator::default());

        Ok(Self {
            stream,
            other_user,
            opening_key,
            sealing_key,
        })
    }

    /// Reads one signed message and decodes it. Just to make other functions in this struct a little smaller and more readable.
    async fn read_signed_message(stream: &mut T) -> Result<SignedMessage, DspfsError> {
        let message = stream.read_with_length_limited(1024).await?;
        Ok(bincode::deserialize(&message)?)
    }

    pub async fn receiver(mut stream: T, user: PrivateUser) -> Result<Self, DspfsError> {
        let signed_init_message = Self::read_signed_message(&mut stream).await?;

        // With the message from the other user, we can start the diffie hellman procedure and
        // create our own shared secret. However, we have to send something back to them so they
        // can get this secret as well
        let init_message: Message = bincode::deserialize(&signed_init_message.message)?;

        let our_partial_secret = EphemeralSecret::new(&mut rand_core::OsRng);
        let our_pubkey = PublicKey::from(&our_partial_secret);

        // Extract message parts and apply diffie hellman to create a shared secret
        let (shared_secret, other_user) = match init_message {
            Message::Init {
                user,
                pubkey: other_pubkey,
            } => (our_partial_secret.diffie_hellman(&other_pubkey), user),
            _ => return Err(DspfsError::InvalidEncryptedConnectionInitialization),
        };

        // Check signature when we know their public key
        // TODO: Don't get this public key from the message but instead from the store.
        other_user
            .get_public_key()
            .ring()
            .verify(&signed_init_message.message, &signed_init_message.signature)
            .map_err(|_| DspfsError::BadSignature)?;

        // Now we know the identity of the other user, send something back
        let our_init_msg = Message::Init {
            user: user.public_user().to_owned(),
            pubkey: our_pubkey,
        };
        let signedmsg = our_init_msg.sign(user.get_keypair())?.serialize()?;
        stream.write_with_length(&signedmsg).await?;

        // TODO: Is this good practice? Should you make a sealing key and opening key from one shared secret?
        //       Should we use different salts for the sealer and opener? Maybe use our username as sealer and
        //       the other user's name as opener?
        let mut salt = other_user.get_username().clone();
        salt.push_str(user.get_username());
        let mut shared_key = [0; 32];
        // TODO: Derive may panic, maybe check for this?
        derive(
            PBKDF2_HMAC_SHA256,
            // DO NOT MAKE 0
            NonZeroU32::new(42u32).unwrap(),
            salt.as_bytes(),
            shared_secret.as_bytes(),
            &mut shared_key,
        );

        // TODO: Is this good practice? Should you make a sealing key and opening key from one shared secret?
        //       Should we use different salts for the sealer and opener? Maybe use our username as sealer and
        //       the other user's name as opener?
        let unbound_key1 = UnboundKey::new(&CHACHA20_POLY1305, &shared_key)?;
        let unbound_key2 = UnboundKey::new(&CHACHA20_POLY1305, &shared_key)?;

        let opening_key = OpeningKey::new(unbound_key1, NonceGenerator::default());
        let sealing_key = SealingKey::new(unbound_key2, NonceGenerator::default());

        Ok(Self {
            stream,
            other_user,
            opening_key,
            sealing_key,
        })
    }

    pub async fn send_message(&mut self, message: Message) -> Result<(), DspfsError> {
        let mut serialized_message = bincode::serialize(&message)?;
        self.sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut serialized_message)?;

        self.stream.write_with_length(&serialized_message).await?;

        Ok(())
    }

    pub async fn recv_message(&mut self, limit: usize) -> Result<Message, DspfsError> {
        let mut msg = self.stream.read_with_length_limited(limit).await?;

        self.opening_key.open_in_place(Aad::empty(), &mut msg)?;

        let dmsg = bincode::deserialize(&msg)?;

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
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
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
    fn test_not_nonce_unqiue() {
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
