use crate::fs::hash::FileHash;
use std::path::PathBuf;

#[derive(Debug)]
pub struct File {
    pub hash: FileHash,
    pub size: u64,
    pub path: PathBuf,
}
