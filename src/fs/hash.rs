use ring::digest::{Digest, SHA512};
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone)]
pub struct FileHash(Digest);

impl<T: AsRef<[u8]>> From<T> for FileHash {
    fn from(file: T) -> Self {
        Self(ring::digest::digest(&SHA512, &file.as_ref()))
    }
}

impl FileHash {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Hash for FileHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state)
    }
}

impl PartialEq for FileHash {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl Eq for FileHash {}
