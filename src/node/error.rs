use std::io;

#[derive(Debug)]
pub enum NodeError {
    IoError(io::Error),
    Error(String)
}

impl From<io::Error> for NodeError {
    fn from(e: io::Error) -> Self {
        NodeError::IoError(e)
    }
}

impl From<String> for NodeError {
    fn from(e: String) -> Self {
        NodeError::Error(e)
    }
}

impl From<&str> for NodeError {
    fn from(e: &str) -> Self {
        NodeError::Error(e.into())
    }
}
