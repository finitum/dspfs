use crate::fs::local::ops::FileOps;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::fs;
use std::io::SeekFrom;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use walkdir::WalkDir;

pub struct RealFileOps;

#[async_trait]
impl FileOps for RealFileOps {
    async fn read(path: &PathBuf, offset: u64, length: u64) -> Result<Vec<u8>> {
        let mut file = File::open(path).await?;
        file.seek(SeekFrom::Start(offset)).await?;

        let mut buffer = vec![0u8; length as usize];

        file.read_exact(&mut buffer).await?;

        Ok(buffer)
    }

    async fn size(path: &PathBuf) -> Result<u64> {
        Ok(path.metadata()?.len())
    }

    async fn write(path: &PathBuf, offset: u64, data: Vec<u8>) -> Result<()> {
        let mut file = File::open(path).await.context("couldn't open file")?;

        // TODO: Fill file with zeros if offset > file size
        // TODO: I hope write_all appends after the position that was seeked to, otherwise this doesn't work
        file.seek(SeekFrom::Start(offset)).await?;

        file.write_all(&data).await?;

        Ok(())
    }

    async fn delete(path: &PathBuf) -> Result<()> {
        fs::remove_file(path)
            .context("couldn't remove file")
            .unwrap();
        Ok(())
    }

    async fn recursive_files(path: &PathBuf) -> Result<Box<dyn Iterator<Item = PathBuf>>> {
        Ok(Box::new(
            WalkDir::new(path)
                .into_iter()
                .filter_map(|i| i.ok())
                .filter(|i| !i.file_type().is_dir())
                .map(|i| i.path().to_path_buf()),
        ))
    }
}
