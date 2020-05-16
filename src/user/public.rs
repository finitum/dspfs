use crate::user::PublicKey;
use std::net::IpAddr;

type SymmetricKey = u8;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PublicUser {
    // ed25519 public key
    public_key: PublicKey,
    username: String,
    last_ip: Option<IpAddr>,
}

impl PublicUser {
    pub fn new(public_key: PublicKey, username: &str) -> Self {
        Self {
            public_key,
            username: username.into(),
            last_ip: None,
        }
    }

    pub fn get_public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn get_username(&self) -> &String {
        &self.username
    }
}
