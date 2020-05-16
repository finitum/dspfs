use crate::error::DspfsError;
use crate::message::{Message, SignedMessage};
use crate::user::{PrivateUser, PublicUser};
use ring::aead::{
    BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, CHACHA20_POLY1305,
    NONCE_LEN,
};
use ring::error::Unspecified;
use ring::pbkdf2::*;
use ring::rand::{SecureRandom, SystemRandom};
use ring::signature::Ed25519KeyPair;
use std::num::NonZeroU32;
use std::task::Context;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::macros::support::{Pin, Poll};
use tokio::net::TcpStream;
use tokio::prelude::AsyncRead;
use x25519_dalek::EphemeralSecret;
use x25519_dalek::PublicKey;

pub struct EncryptedStream {
    stream: TcpStream,
    other_user: PublicUser,

    opening_key: OpeningKey<NonceGenerator>,
    sealing_key: SealingKey<NonceGenerator>,
}

impl EncryptedStream {
    pub async fn initiator(mut stream: TcpStream, user: PrivateUser) -> Result<Self, DspfsError> {
        // TODO: Maybe switch to ring for DH
        // TODO: Refactor to be independent from TcpStream

        let our_partial_secret = EphemeralSecret::new(&mut rand_core::OsRng);
        let our_pubkey = PublicKey::from(&our_partial_secret);

        let msg = Message::Init {
            user: user.public_user().to_owned(),
            pubkey: our_pubkey,
        };

        let signedmsg = msg.sign(user.get_keypair())?.serialize()?;

        stream.write(&signedmsg).await?;

        let mut reply = Vec::new();
        stream.read_to_end(&mut reply).await?;

        let other_init: SignedMessage = bincode::deserialize(&reply)?;

        let other_user_message: Message = bincode::deserialize(&other_init.message)?;

        let (shared_secret, other_user) = match other_user_message {
            Message::Init {
                user,
                pubkey: other_pubkey,
            } => (our_partial_secret.diffie_hellman(&other_pubkey), user),
            _ => Err(DspfsError::InvalidEncryptedConnectionInitialization)?,
        };

        // TODO: Less hardcoding of thingies

        // WATCH OUT: REVERSE USERNAMES ON RECEIVING SIDE TO MAKE EQUAL SALTS
        let mut salt = user.get_username().clone();
        salt.push_str(other_user.get_username());
        let mut shared_key = Vec::new();
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

        let opening_key = OpeningKey::new(unbound_key1, NonceGenerator);
        let sealing_key = SealingKey::new(unbound_key2, NonceGenerator);

        Ok(Self {
            stream,
            other_user,
            opening_key,
            sealing_key,
        })
    }

    pub fn receiver(_stream: TcpStream, _keypair: Ed25519KeyPair) -> Self {
        let our_partial_secret1 = EphemeralSecret::new(&mut rand_core::OsRng);
        let _our_partial_public1 = PublicKey::from(&our_partial_secret1);

        todo!()
    }
}

impl AsyncWrite for EncryptedStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        unimplemented!()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        unimplemented!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        unimplemented!()
    }
}

impl AsyncRead for EncryptedStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        unimplemented!()
    }
}

struct NonceGenerator;

impl NonceSequence for NonceGenerator {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        // Random data must be used only once per encryption
        let mut nonce = [0u8; NONCE_LEN];

        // Fill nonce with random data
        let rand = SystemRandom::new();
        rand.fill(&mut nonce)?;

        Ok(Nonce::assume_unique_for_key(nonce))
    }
}

#[cfg(test)]
mod tests {
    use crate::encryptedstream::NonceGenerator;
    use ring::aead::NonceSequence;

    #[test]
    fn test_not_0_nonce() {
        let n = NonceGenerator {}.advance().unwrap();

        assert_ne!(*n.as_ref(), [0u8; 12])
    }

    #[test]
    fn test_not_nonce_unqiue() {
        let mut n = NonceGenerator {};
        let a = n.advance().unwrap();
        let b = n.advance().unwrap();

        assert_ne!(a.as_ref(), b.as_ref())
    }
}
