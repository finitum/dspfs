use crate::error::DspfsError;
use crate::store::Store;
use crate::user::{PublicKey, PublicUser};
use ring::pkcs8::Document;
use ring::rand;
use ring::signature::Ed25519KeyPair;
use std::ops::{Deref, DerefMut};

pub struct PrivateUser {
    // The "embeded"public user
    public_user: PublicUser,

    // KeyPair
    keypair: ring::signature::Ed25519KeyPair,
}

impl PrivateUser {
    /// Creates a new user, generates a new keypair for them.
    pub fn new(username: &str) -> Result<(Self, Document), DspfsError> {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;
        let keypair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;

        Ok((
            Self {
                public_user: PublicUser::new(PublicKey::from(&keypair), username),
                keypair,
            },
            pkcs8_bytes,
        ))
    }

    pub fn get_keypair(&self) -> &Ed25519KeyPair {
        &self.keypair
    }

    pub fn public_user(&self) -> &PublicUser {
        &self.public_user
    }

    pub fn load_from_store(store: &dyn Store) -> Result<Self, DspfsError> {
        Ok(Self {
            public_user: store
                .get_self_user()
                .clone()
                .ok_or(DspfsError::NotFoundInStore(
                    "Could not load private user from store: Self user not found".into(),
                ))?,
            keypair: store.get_signing_key().ok_or(DspfsError::NotFoundInStore(
                "Could not load private user from store: Self user not found".into(),
            ))??,
        })
    }
}

impl Deref for PrivateUser {
    type Target = PublicUser;

    fn deref(&self) -> &Self::Target {
        &self.public_user
    }
}

impl DerefMut for PrivateUser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.public_user
    }
}
