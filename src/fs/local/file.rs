use crate::fs::hash::FileHash;
use std::path::PathBuf;

pub struct File {
    pub hash: FileHash,
    pub size: u64,
    pub path: PathBuf
}
