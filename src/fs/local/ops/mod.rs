use crate::fs::hash::FileHash;
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

pub mod nop;
pub mod real;

#[async_trait]
pub trait FileOps: Send + Sync {
    async fn read(path: &PathBuf, offset: u64, length: u64) -> Result<Vec<u8>>;

    async fn size(path: &PathBuf) -> Result<u64>;

    async fn write(path: &PathBuf, offset: u64, data: Vec<u8>) -> Result<()>;

    async fn delete(path: &PathBuf) -> Result<()>;

    async fn hash(path: &PathBuf) -> Result<FileHash> {
        let file = Self::read(&path, 0, Self::size(&path).await?).await?;

        Ok(FileHash::from(file))
    }

    /// Returns an iterator over all files in a directory, recursively
    async fn recursive_files(path: &PathBuf) -> Result<Box<dyn Iterator<Item = PathBuf>>>;
}
