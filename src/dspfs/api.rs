use crate::fs::hash::Hash;
use crate::user::PublicUser;

use crate::fs::file::SimpleFile;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::ffi::OsString;
use std::fs::DirEntry;
use std::path::Path;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct LocalFile {
    name: OsString,
    modtime: SystemTime,
    is_dir: bool,
    file_size: u64,
}

impl LocalFile {
    pub fn from_direntry(direntry: DirEntry) -> Result<Self> {
        let metadata = direntry.metadata()?;

        Ok(Self {
            name: direntry.file_name(),
            modtime: metadata.modified()?,
            is_dir: metadata.is_dir(),
            file_size: metadata.len(),
        })
    }
}

#[async_trait]
pub trait Api {
    /// Equivalent of `git init`. Creates a new group with only you in it.
    async fn init_group(&self, path: &Path) -> Result<Uuid>;

    /// Equivalent of `git add`. Adds a file to the group and makes it visible for others in the group.
    async fn add_file(&self, guuid: Uuid, path: &Path) -> Result<SimpleFile>;

    /// Bootstraps a group.
    async fn join_group(&self, guuid: Uuid, bootstrap_user: &PublicUser) -> Result<()>;

    /// Gives a flat list of all files and their hashes.
    async fn list_files(&self, guuid: Uuid) -> Result<HashSet<SimpleFile>>;

    /// Gets all users present in the group (that we know of).
    async fn get_users(&self, guuid: Uuid) -> Result<BTreeSet<PublicUser>>;

    /// gives a list of files in your local filesystem, which you can share in the group
    async fn get_available_files(&self, guuid: Uuid, path: &Path) -> Result<HashSet<LocalFile>>;

    /// gets a certain level of filetree as seen by another user
    async fn get_files(
        &self,
        guuid: Uuid,
        user: &PublicUser,
        path: &Path,
    ) -> Result<HashSet<SimpleFile>>;

    /// requests a file from other users in the group
    async fn request_file(&self, hash: Hash, to: &Path) -> Result<()>;

    /// Lists current download/uploads.
    async fn status(&self) {
        todo!()
    }

    /// Refreshes internal state.
    /// may do any of the following things:
    ///  * Re-index local files.
    ///  * Check for new users in the group.
    ///  * Check for new files from other users.
    async fn refresh(&self);

    // TODO:
    async fn add_folder(&self, _guuid: Uuid, path: &Path) -> Result<()> {
        assert!(path.is_dir());
        todo!()
    }
}
