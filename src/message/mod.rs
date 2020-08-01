use crate::user::PublicUser;
use anyhow::{Context, Result};
use ring::signature::Ed25519KeyPair;
use std::fmt::Debug;
use uuid::Uuid;
use crate::fs::hash::Hash;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ErrorMessage {
    /// Someone asked for this file but we don't have it
    FileNotFound
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Message {
    Init { user: PublicUser, pubkey: Vec<u8> },
    String(String),
    FileBlockRequest {
        groupuuid: Uuid,
        filehash: Hash,
        index: u64,
    },
    
    // Returns a file requested by a file request
    FileBlock(Vec<u8>),

    // Something went wrong!
    Error(ErrorMessage)
}

impl Message {
    pub fn sign(&self, keypair: &Ed25519KeyPair) -> Result<SignedMessage> {
        let message = bincode::serialize(self).context("failed to serialize message")?;
        let signature = keypair.sign(&message).as_ref().to_vec();

        Ok(SignedMessage { message, signature })
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self).context("failed to deserialize message")?)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SignedMessage {
    pub message: Vec<u8>, // a serialized Message
    pub signature: Vec<u8>,
}

impl SignedMessage {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self).context("failed to serialize message")?)
    }
}
