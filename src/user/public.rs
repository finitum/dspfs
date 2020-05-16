use crate::user::PublicKey;
use std::net::IpAddr;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PublicUser {
    // ed25519 public key
    public_key: PublicKey,
    // username(?)
    username: String,
    // Option<Last known ip>
    last_ip: Option<IpAddr>, // Option<symmetric key> (when established)
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
