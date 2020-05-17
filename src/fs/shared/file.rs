use crate::fs::hash::FileHash;
use std::path::PathBuf;
use crate::fs::local;

pub struct File {
    hash: FileHash,
    size: u64,
    path: PathBuf
}

impl From<&local::File> for File {
    fn from(f: &local::File) -> Self {
        Self {
            hash: f.hash,
            size: f.size,
            path: f.path.clone()
        }
    }
}