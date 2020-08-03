use crate::fs::file::File;
use crate::fs::filetree::FileTree;
use crate::fs::hash::Hash;
use crate::user::PublicUser;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Thread safe group store
pub type SharedGroupStore = Arc<RwLock<Box<dyn GroupStore>>>;

/// GroupStore maintains the map of all files available and who owns what file where
///
/// This could be an example of its internal structure:
/// ```ignore
/// struct GroupStoreImpl {
///   filetrees: Map<User, Filetree>,
///   files: Map<Hash, File>,
/// }
/// ```
pub trait GroupStore: Send + Sync {
    /// Adds a file to a user
    fn add_file(&mut self, user: &PublicUser, file: File) -> Result<()>;

    /// Gets a specific file given a filehash
    fn get_file(&self, hash: Hash) -> Result<Option<File>>;

    /// Gets the list of all files
    fn list_files(&self) -> Result<Vec<File>>;

    /// Gets the file tree of a specific user
    fn get_filetree(&self, user: &PublicUser) -> Result<FileTree>;

    /// Changes a user's file from old to new.
    fn update_file(&mut self, user: &PublicUser, old: &File, new: File) -> Result<()> {
        self.delete_file(user, old)?;
        self.add_file(user, new)
    }

    /// Deletes a file from a user's file tree, and updates who has this file.
    /// Errors if the file did not exist.
    fn delete_file(&mut self, user: &PublicUser, file: &File) -> Result<()>;
}
