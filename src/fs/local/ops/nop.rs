use crate::fs::local::ops::FileOps;
use anyhow::Result;
use async_trait::async_trait;
use log::*;
use std::iter;
use std::path::PathBuf;

pub struct NopFileOps;

/// Warning: this may print file contents to stdout
#[async_trait]
impl FileOps for NopFileOps {
    async fn read(path: &PathBuf, offset: u64, length: u64) -> Result<Vec<u8>> {
        info!("Reading {:?} from {} to {}", path, offset, offset + length);
        Ok(vec![0; length as usize])
    }

    async fn size(_path: &PathBuf) -> Result<u64> {
        Ok(0)
    }

    async fn write(path: &PathBuf, offset: u64, data: Vec<u8>) -> Result<()> {
        info!("Writing {:?} to {:?} at {}", data, path, offset);
        Ok(())
    }

    async fn delete(path: &PathBuf) -> Result<()> {
        info!("Deleting {:?}", path);
        Ok(())
    }

    async fn recursive_files(path: &PathBuf) -> Result<Box<dyn Iterator<Item = PathBuf>>> {
        info!("Searching files in {:?}", path);

        Ok(Box::new(iter::empty()))
    }
}
