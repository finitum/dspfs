use crate::fs::hash::{Hash, HashingAlgorithm, BLOCK_HASHING_ALGORITHM};
use crate::user::PublicUser;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::fs::File as tFile;
use tokio::io::AsyncReadExt;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct File {
    /// filename/location locally relative to group root.
    /// If this variant is None, then this user is guaranteed not to be in the file's user list.
    pub path: PathBuf,

    /// Hash of the entire file
    pub hash: Hash,

    pub hashing_algorithm: HashingAlgorithm,

    /// The size of each block in this file
    /// This size should never be changed after this file has been created.
    /// If it does, the file will be recognized as a different file with a different hash
    pub(crate) block_size: u64,

    /// Hashes of each block in the file. In the same order as the blocks appear in the file.
    blockhashes: Vec<Hash>,

    /// People who are likely to have the file. This set is added to whenever
    /// we learn that someone has a file, either from them directly or from someone else.
    /// Only when we ask this user for the file and it turns out they don't have it anymore,
    /// do we remove him from this set.
    users: HashSet<PublicUser>,
    // TODO:
    // modtime
}

// All operations on files are immutable and return a new file.
// This is to force you to read it to the store
impl File {
    /// new_empty creates a new File from a path as if the file is empty
    pub(crate) fn new_empty(path: PathBuf) -> Self {
        let block_hash = Hash::hash_block(BLOCK_HASHING_ALGORITHM, &[]);
        let file_hash =
            Hash::hash_block_hashes(BLOCK_HASHING_ALGORITHM, vec![block_hash.clone()].as_ref());

        Self {
            path,
            hash: file_hash,
            hashing_algorithm: BLOCK_HASHING_ALGORITHM,
            block_size: block_size(0),
            blockhashes: vec![block_hash],
            users: HashSet::new(),
        }
    }

    /// new creates a new File from a path, this will calculate all appropriate hashes and other
    /// relevant metadata.
    pub async fn new(path: PathBuf) -> Result<Self> {
        // 1. Open file
        let mut file = tFile::open(&path).await.context("Couldn't open file")?;

        // 2. Divide into blocks
        let metadata = file.metadata().await.context("Couldn't access metadata")?;

        // 2.1 determine block size
        let file_size = metadata.len();
        if file_size == 0 {
            return Ok(Self::new_empty(path));
        }

        // 3-4
        let block_size = block_size(file_size);
        let (file_hash, block_hashes) = Self::hash_file(&mut file, block_size).await?;

        // 5. Create File
        Ok(Self {
            path,
            hash: file_hash,
            hashing_algorithm: BLOCK_HASHING_ALGORITHM,
            block_size,
            blockhashes: block_hashes,
            users: Default::default(),
        })
    }

    /// Hashes a filesystem file given a specified block_size, returns hash and block_level hashes
    async fn hash_file(file: &mut tFile, block_size: u64) -> Result<(Hash, Vec<Hash>)> {
        let file_size = file
            .metadata()
            .await
            .context("retrieving metadata failed")?
            .len();
        // 1 + ((x - 1) / y)
        let numblocks = 1 + ((file_size - 1) / block_size);
        let last_block_len = file_size - ((numblocks - 1) * block_size);

        // 3. Hash each block
        let mut buffer = vec![0u8; block_size as usize];
        let mut block_hashes = Vec::with_capacity(numblocks as usize);

        for _ in 0..(numblocks - 1) {
            file.read_exact(&mut buffer)
                .await
                .context("reading block from file failed")?;
            block_hashes.push(Hash::hash_block(BLOCK_HASHING_ALGORITHM, &buffer));
        }

        let mut buffer = vec![0u8; last_block_len as usize];
        file.read_exact(&mut buffer)
            .await
            .context("reading block from file failed")?;
        block_hashes.push(Hash::hash_block(BLOCK_HASHING_ALGORITHM, &buffer));

        if block_hashes.len() as u64 != numblocks {
            return Err(anyhow::anyhow!(
                "number of hashes mismatches of number of blocks"
            ));
        }

        // 4. hash blockhashes for file hash
        let file_hash = Hash::hash_block_hashes(BLOCK_HASHING_ALGORITHM, &block_hashes);
        Ok((file_hash, block_hashes))
    }

    /// Rehashes this file, to be used if the file changes.
    pub async fn rehash(&mut self) -> Result<()> {
        // 1. Open File
        let mut file = tFile::open(&self.path)
            .await
            .context("opening file failed")?;
        // 2. Call hash_file
        let (file_hash, block_hashes) = Self::hash_file(&mut file, self.block_size).await?;
        // 3. save new info
        self.hash = file_hash;
        self.blockhashes = block_hashes;

        Ok(())
    }

    pub fn is_owned_by(&self, user: &PublicUser) -> bool {
        self.users.contains(user)
    }

    pub fn num_owning_users(&self) -> usize {
        self.users.len()
    }

    /// Returns true if two file structs refer to the same file.
    /// This is the case when the file hashes ar equal.
    pub fn equals(&self, other: &File) -> bool {
        self.hash == other.hash
    }

    pub fn merge_users(&mut self, file: &File) {
        self.users = self.users.union(&file.users).cloned().collect();
    }

    pub fn remove_user(&mut self, user: &PublicUser) -> bool {
        self.users.remove(user)
    }

    pub fn get_block_hash(&self, index: u64) -> Option<&Hash> {
        self.blockhashes.get(index as usize)
    }
}

/// Based on syncthing's [BEP](https://docs.syncthing.net/specs/bep-v1.html#blocksize)
fn block_size(len: u64) -> u64 {
    match len {
        // 0 - 256 MiB => 128 KiB
        0..=268435456 => 128 * 1024,
        // 256 - 512 MiB => 256 KiB
        268435457..=536870912 => 256 * 1024,
        // 512 - 1024 MiB => 512 KiB
        536870913..=1073741824 => 512 * 1024,
        // 1GiB - 2 GiB => 1 MiB
        1073741825..=2147483649 => 1024 * 1024,
        // 2GiB - 4 GiB => 2 MiB
        2147483650..=4294967296 => 1024 * 1024 * 2,
        // 4 GiB - 8 GiB => 4 MiB
        4294967297..=8589934592 => 1024 * 1024 * 4,
        // 8GiB - 16 GiB => 8 MiB
        8589934593..=17179869184 => 1024 * 1024 * 8,
        // 16GiB - up => 16 MiB
        _ => 1024 * 1024 * 16,
    }
}
