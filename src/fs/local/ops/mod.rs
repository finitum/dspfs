use crate::error::DspfsError;
use std::path::PathBuf;
use async_trait::async_trait;
use crate::fs::hash::FileHash;

mod nop;

#[async_trait]
pub trait FileOps: Send + Sync {
    async fn read(
        path: &PathBuf,
        offset: usize,
        length: usize,
    ) -> Result<Vec<u8>, DspfsError>;

    async fn size(path: &PathBuf) -> Result<u64, DspfsError>;

    async fn write(
        path: &PathBuf,
        offset: usize,
        data: Vec<u8>,
    ) -> Result<(), DspfsError>;

    async fn delete(path: &PathBuf) -> Result<(), DspfsError>;

    async fn hash(
        path: &PathBuf,
    ) -> Result<FileHash, DspfsError> {
        let file = Self::read(&path, 0, Self::size(&path).await? as usize).await?;

        Ok(FileHash::from(file))
    }
}
