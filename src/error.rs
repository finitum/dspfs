use serde::export::fmt::Debug;
use std::io;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug)]
pub enum DspfsError {
    IoError(io::Error),
    Error(String),
    NotFoundInStore(String),
    RingKeyRejectedError(ring::error::KeyRejected),
    RingKeyUnspecifiedError(ring::error::Unspecified),
    InvalidEncryptedConnectionInitialization,
    BincodeError(Box<bincode::ErrorKind>),
    BadSignature,
    InvalidMessage,
    ChannelSendError(String),
}

impl From<io::Error> for DspfsError {
    fn from(e: io::Error) -> Self {
        DspfsError::IoError(e)
    }
}

impl From<String> for DspfsError {
    fn from(e: String) -> Self {
        DspfsError::Error(e)
    }
}

impl From<&str> for DspfsError {
    fn from(e: &str) -> Self {
        DspfsError::Error(e.into())
    }
}

impl From<ring::error::KeyRejected> for DspfsError {
    fn from(e: ring::error::KeyRejected) -> Self {
        DspfsError::RingKeyRejectedError(e)
    }
}

impl From<ring::error::Unspecified> for DspfsError {
    fn from(e: ring::error::Unspecified) -> Self {
        DspfsError::RingKeyUnspecifiedError(e)
    }
}

impl From<Box<bincode::ErrorKind>> for DspfsError {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        DspfsError::BincodeError(e)
    }
}

impl<T> From<SendError<T>> for DspfsError {
    fn from(e: SendError<T>) -> Self {
        DspfsError::ChannelSendError(e.to_string())
    }
}
