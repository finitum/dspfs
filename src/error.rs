use std::io;

#[derive(Debug)]
pub enum DspfsError {
    IoError(io::Error),
    Error(String),
    NotFoundInStore(String),
    RingKeyRejectedError(ring::error::KeyRejected),
    RingKeyUnspecifiedError(ring::error::Unspecified),
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
