use crate::fs::local::file::File;
use crate::fs::shared;
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use crate::error::DspfsError;
use crate::fs::hash::FileHash;
use crate::fs::local::ops::FileOps;
use std::marker::PhantomData;
use serde::export::fmt::Debug;
use crate::store::SharedStore;

/// Local representation of a group
pub struct Group<F: FileOps> {
    files: HashMap<PathBuf, File>,

    /// Views
    hashes: HashMap<FileHash, PathBuf>,

    shared_group: shared::Group,

    phantom: PhantomData<F>,
}


impl<F: FileOps> Group<F> {
    pub async fn add_file(&mut self, file: File, store: SharedStore) -> Result<(), DspfsError> {
        let guard = store.read().await;
        let me = guard.get_self_user()
            .as_ref()
            .ok_or(DspfsError::NotFoundInStore("Group::add_file(): Could not find user in store".into()))?;

        self.shared_group.add_file(&me, shared::File::from(&file)).await;

        self.hashes.insert(file.hash.clone(), file.path.clone());
        self.files.insert(file.path.clone(), file);

        Ok(())
    }

    pub async fn add_file_from_path(&mut self, file: impl AsRef<Path> + Debug + Send + Sync, store: SharedStore) -> Result<(), DspfsError> {
        let path = file.as_ref().to_path_buf();
        let hash = F::hash(&path).await?;
        let size = F::size(&path).await?;

        self.add_file(File {
            hash,
            size,
            path,
        }, store).await?;

        Ok(())
    }
}