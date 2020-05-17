use crate::fs::hash::FileHash;
use crate::fs::local;
use std::path::PathBuf;

#[cfg(not(test))]
#[derive(Clone, Debug, PartialEq)]
pub struct File {
    fhash: FileHash,
    size: u64,
    path: PathBuf,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub struct File {
    pub(crate) fhash: FileHash,
    pub(crate) size: u64,
    pub(crate) path: PathBuf,
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
