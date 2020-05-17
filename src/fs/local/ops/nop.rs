use crate::error::DspfsError;
use crate::fs::local::ops::FileOps;
use async_trait::async_trait;
use log::*;
use std::path::PathBuf;

struct NopFileOps;

/// Warning: this may print file contents to stdout
#[async_trait]
impl FileOps for NopFileOps {
    async fn read(path: &PathBuf, offset: usize, length: usize) -> Result<Vec<u8>, DspfsError> {
        info!("Reading {:?} from {} to {}", path, offset, offset + length);
        Ok(vec![0; length])
    }

    async fn size(_path: &PathBuf) -> Result<u64, DspfsError> {
        Ok(0)
    }

    async fn write(path: &PathBuf, offset: usize, data: Vec<u8>) -> Result<(), DspfsError> {
        info!("Writing {:?} to {:?} at {}", data, path, offset);
        Ok(())
    }

    async fn delete(path: &PathBuf) -> Result<(), DspfsError> {
        info!("Deleting {:?}", path);
        Ok(())
    }
}
