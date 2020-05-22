use crate::fs::hash::FileHash;
use crate::fs::local;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct File {
    pub(super) fhash: FileHash,
    pub(super) size: u64,
    pub(super) path: PathBuf,
}

impl File {
    pub fn get_filehash(&self) -> FileHash {
        self.fhash
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
}

impl From<&local::File> for File {
    fn from(f: &local::File) -> Self {
        Self {
            fhash: f.hash,
            size: f.size,
            path: f.path.clone(),
        }
    }
}
