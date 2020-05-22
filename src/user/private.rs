use crate::store::Store;
use crate::user::PublicUser;
use anyhow::{Context, Result};
use ring::pkcs8::Document;
use ring::rand;
use ring::signature::Ed25519KeyPair;
use std::convert::TryInto;
use std::ops::{Deref, DerefMut};

pub struct PrivateUser {
    // The "embeded"public user
    public_user: PublicUser,

    // KeyPair
    keypair: ring::signature::Ed25519KeyPair,
}

impl PrivateUser {
    /// Creates a new user, generates a new keypair for them.
    pub fn new(username: &str) -> Result<(Self, Document)> {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|_| anyhow::anyhow!("unspecified ring error"))?;
        let keypair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|_| anyhow::anyhow!("key rejected error"))?;

        Ok((
            Self {
                public_user: PublicUser::new((&keypair).try_into()?, username),
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

    pub fn load_from_store<S: Store>(store: &S) -> Result<Self> {
        Ok(Self {
            public_user: store
                .get_me()
                .context("couldn't load from store")?
                .context("user not found in store")?,
            keypair: store
                .get_signing_key()
                .context("couldn't load key from store")?
                .context("signing key not present int store")?,
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
