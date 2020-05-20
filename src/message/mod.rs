use crate::error::DspfsError;
use crate::user::PublicUser;
use ring::signature::Ed25519KeyPair;
use std::fmt::Debug;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Message {
    Init { user: PublicUser, pubkey: Vec<u8> },
    String(String),
}

impl Message {
    pub fn sign(&self, keypair: &Ed25519KeyPair) -> Result<SignedMessage, DspfsError> {
        let message = bincode::serialize(self)?;
        let signature = keypair.sign(&message).as_ref().to_vec();

        Ok(SignedMessage { message, signature })
    }

    pub fn serialize(&self) -> Result<Vec<u8>, DspfsError> {
        Ok(bincode::serialize(self)?)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SignedMessage {
    pub message: Vec<u8>, // a serialized Message
    pub signature: Vec<u8>,
}

impl SignedMessage {
    pub fn serialize(&self) -> Result<Vec<u8>, DspfsError> {
        Ok(bincode::serialize(self)?)
    }
}
