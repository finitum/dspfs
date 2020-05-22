use crate::dspfs::notify::Notify;
use crate::fs::hash::FileHash;
use crate::fs::local::file::File;
use crate::fs::local::ops::FileOps;
use crate::fs::shared;
use crate::store::{SharedStore, Store};
use anyhow::{Context, Result};
use log::*;
use serde::export::fmt::Debug;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Local representation of a group
pub struct Group {
    files: HashMap<PathBuf, File>,

    hashes: HashMap<FileHash, PathBuf>,

    shared_group: shared::Group,

    /// This is the location in the user's filesystem where the group is attached to.
    /// Optionally None, if this Group is not attached to any folder.
    path: Option<PathBuf>,
}

impl Group {
    pub async fn new<S: Store>(store: SharedStore<S>) -> Result<Self> {
        let store = store.read().await;
        let me = store.get_me().context("")?.context("")?;
        let res = Self {
            files: HashMap::new(),
            hashes: HashMap::new(),
            shared_group: shared::Group::new(me),
            path: None,
        };

        Ok(res)
    }

    pub async fn attach<F, P, S: Store>(
        &mut self,
        path: P,
        store: SharedStore<S>,
        notifier: &mut dyn Notify,
    ) -> Result<()>
    where
        F: FileOps,
        P: AsRef<Path>,
    {
        self.path = Some(path.as_ref().to_path_buf());
        self.index::<F, _>(store, notifier).await?;
        Ok(())
    }

    pub async fn index<F: FileOps, S: Store>(
        &mut self,
        store: SharedStore<S>,
        notifier: &mut dyn Notify,
    ) -> Result<()> {
        // TODO: Check which files were already indexed
        if let Some(ref path) = self.path {
            for entry in F::recursive_files(path).await? {
                debug!("Adding file: {:?}", entry);
                self.add_file_from_path::<F, _, _>(entry, store.clone(), notifier)
                    .await?
            }
        }
        Ok(())
    }

    pub fn is_attached(&self) -> bool {
        self.path.is_some()
    }

    pub async fn add_file<S: Store>(
        &mut self,
        file: File,
        store: SharedStore<S>,
        notifier: &mut dyn Notify,
    ) -> Result<()> {
        let store = store.read().await;
        let me = store
            .get_me()
            .context("Group::add_file(): Could not access the store")?
            .context("Group::add_file(): Could not find user in store")?;

        let shared_file = shared::File::from(&file);
        notifier.file_added(&shared_file).await?;
        self.shared_group.add_file(&me, shared_file);

        self.hashes.insert(file.hash, file.path.clone());
        self.files.insert(file.path.clone(), file);

        Ok(())
    }

    pub async fn add_file_from_path<F, P, S: Store>(
        &mut self,
        file: P,
        store: SharedStore<S>,
        notifier: &mut dyn Notify,
    ) -> Result<()>
    where
        F: FileOps,
        P: AsRef<Path> + Debug + Send + Sync,
    {
        let path = file.as_ref().to_path_buf();
        let hash = F::hash(&path).await?;
        let size = F::size(&path).await?;

        self.add_file(File { hash, size, path }, store, notifier)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dspfs::notify::mock::NotifyMock;
    use crate::fs::local::ops::real::RealFileOps;
    use crate::fs::local::Group;
    use crate::init;
    use crate::store::inmemory::InMemoryStore;

    #[tokio::test]
    pub async fn test_attach() {
        init();

        let store = InMemoryStore::test_store("Test").unwrap();

        let mut group = Group::new(store.clone()).await.unwrap();

        let mut notifier = NotifyMock::new();

        group
            .attach::<RealFileOps, _, _>("./src", store.clone(), &mut notifier)
            .await
            .unwrap();

        dbg!(&group.files);
    }
}
