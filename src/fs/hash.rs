use serde::{Serialize, Deserialize};

pub const BLOCK_HASHING_ALGORITHM: HashingAlgorithm = HashingAlgorithm::BLAKE3;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum HashingAlgorithm {
    BLAKE3
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct Hash {
    hash: Vec<u8>,
}

impl Hash {
    pub fn new(hash: Vec<u8>) -> Self {
        Self { hash }
    }

    pub fn hash_block(algorithm: HashingAlgorithm, block: &[u8]) -> Self {
        match algorithm{
            HashingAlgorithm::BLAKE3 => Self::new(blake3::hash(block).as_bytes().to_vec())
        }
    }

    pub fn hash_block_hashes(algorithm: HashingAlgorithm, block_hashes: &[Hash]) -> Self {
        match algorithm {
            HashingAlgorithm::BLAKE3 => {
                let mut hasher = blake3::Hasher::new();
                for hash in block_hashes {
                    hasher.update(hash.bytes());
                }
                Self::new(hasher.finalize().as_bytes().to_vec())
            }
        }
    }

    pub fn bytes(&self) -> &[u8] {
        self.hash.as_slice()
    }
}
