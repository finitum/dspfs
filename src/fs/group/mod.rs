mod heed;
mod store;

use crate::fs::file::{File, SimpleFile};
use crate::fs::filetree::FileTree;
use crate::fs::group::heed::HeedGroupStore;
use crate::fs::group::store::{GroupStore, SharedGroupStore};
use crate::fs::hash::Hash;
use crate::global_store::{SharedStore, Store};
use crate::user::PublicUser;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::io::SeekFrom;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A *StoredGroup* is a reduced version of a [Group], which can safely be stored in a database.
/// For documentation on what a DSPFS *Group* is, refer to the documentation of [Group].
/// A stored group can be *reloaded* to allow it to be used as a regular [Group] again. Only regular
/// groups can perform actions that need to be reflected in the database. Some actions require the
/// database to be available, and these actions are therefore only available on loaded [Group]s.
///
/// All operations supported on *StoredGroup*s are also available on loaded [Group]s.
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredGroup {
    pub uuid: Uuid,
    pub users: BTreeSet<PublicUser>,
    pub location: PathBuf,
}

impl StoredGroup {
    /// Make sure that `path` points to a valid dspfs folder structure (including a .dspfs subfolder)
    /// Without this, calling `reload()` will error. To make sure you are making a group correctly,
    /// use [Dspfs::new_group()](crate::dspfs::Dspfs::new_group)
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            users: BTreeSet::new(),
            location: path.as_ref().to_path_buf(),
        }
    }

    /// Creates a new full group from a stored group
    /// TODO: cache
    pub fn reload<S: Store>(self, global_store: SharedStore<S>) -> Result<Group<S>> {
        Group::from_stored(self, global_store)
    }

    pub fn dspfs_folder(&self) -> PathBuf {
        self.location.join(".dspfs")
    }

    pub fn dspfs_root(&self) -> &Path {
        self.location.as_path()
    }
}

/// A Group is a structure which represents a folder on your computer which is shared through DSPFS.
/// A group is attached to a folder in your local filesystem, and stores it's data in the `.dspfs`
/// folder located in the folder the group is attached to. This folder is also called the root of a
/// dspfs group.
///
/// A dspfs group structure can be either loaded or stored. Only loaded groups can perform actions
/// concerning the database. For more information about stored groups, refer to the documentation of
/// the [StoredGroup] struct.
///
/// Each group can have a number of members. Members all have their own computers, and all files and
/// folders withing a dspfs group, are shared with all other members of that group. A group
/// is attached to a folder in your local filesystem, and files and folders below this attachment point
/// can be shared using dspfs. These files do not necessarily have to have the same name and location in
/// your filesystem, as other members in the group have that same file in their filesystem. However,
/// using dspfs it is possible to browse other peoples filesystem (the folder they attached their group to)
/// and to download files you want.
///
/// Warning: files you add to the dspfs group can be seen by other people in the group. This is the
/// purpose of dspfs. However, make sure not to put information in a dspfs group you like to
/// remain secret from other members in the group.
/// TODO: It will be possible to exclude folders in this directory from being shared.
#[derive(Clone)]
pub struct Group<S> {
    pub stored_group: StoredGroup,

    group_store: SharedGroupStore,
    global_store: SharedStore<S>,
}

impl<S: Store> Group<S> {
    /// Opens the database in the database folder. Creates it if it didn't exist.
    fn open_db(db_folder: impl AsRef<Path>) -> Result<Arc<RwLock<Box<dyn GroupStore>>>> {
        enum DbType {
            Heed,
        }

        // Somehow detect db type automatically
        let db_type = DbType::Heed;

        Ok(Arc::new(RwLock::new(Box::new(match db_type {
            DbType::Heed => HeedGroupStore::new(db_folder.as_ref().join("heed.mdb"))?,
        }))))
    }

    fn from_stored(stored_group: StoredGroup, global_store: SharedStore<S>) -> Result<Self> {
        Ok(Self {
            group_store: Self::open_db(stored_group.dspfs_folder())?,
            stored_group,
            global_store,
        })
    }

    // /// Sets the filetree received from a user in the group
    // async fn set_filetree(&mut self, user: &PublicUser, filetree: FileTree) -> Result<()>{
    //     for i in filetree.iter() {
    //         if let Some(f) = self.get_file_by_hash(i.hash)? {
    //             // merge the sets in these files
    //             let new = f.merge_users(i);
    //             self.update_file(&f, new)?;
    //         } else {
    //             self.add_remote_file(i.clone())?;
    //         }
    //     }
    //
    //     self.store.read().await.set_filetree(user, filetree)
    // }

    /// Adds a file to the group that exists locally on your filesystem.
    /// This function will hash the file, create a [File] struct and insert it.
    pub async fn index_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let self_user = self
            .global_store
            .read()
            .await
            .get_self_user()
            .context("Could not get self user due to database error")?
            .context("Could not get self user")?;

        let file = File::new(path.as_ref().to_path_buf())
            .await
            .context("Creating and indexing new file failed")?;

        self.add_file(&self_user, file).await
    }

    /// [add_file] adds a file to the relevant databases.
    /// This is the same as saying that we _know_ about this file.
    pub async fn add_file(&mut self, user: &PublicUser, file: File) -> Result<()> {
        self.group_store
            .write()
            .await
            .add_file(user, file)
            .context("adding file to database went wrong")
    }

    pub async fn get_local_file(&self, hash: Hash) -> Result<Option<File>> {
        let self_user = self
            .global_store
            .read()
            .await
            .get_self_user()
            .context("Could not get self user due to database error")?
            .context("Could not get self user")?;

        Ok(self
            .group_store
            .read()
            .await
            .get_file(hash)?
            .map(|f| {
                if f.is_owned_by(&self_user) {
                    None
                } else {
                    Some(f)
                }
            })
            .flatten())
    }

    pub async fn get_block_contents(&self, hash: Hash, index: u64) -> Result<Option<Vec<u8>>> {
        let file = if let Some(f) = self.get_local_file(hash).await? {
            f
        } else {
            return Ok(None);
        };

        let mut path = self
            .dspfs_folder()
            .parent()
            .context("invalid full dspfs folder path")?
            .to_path_buf();
        path.push(&file.path);

        // open file
        let mut open_file = fs::File::open(path).await.context("failed to open file")?;

        // seek block start
        open_file
            .seek(SeekFrom::Start(index * file.block_size))
            .await
            .context("this block doesn't exist in this file")?;

        let mut buffer = vec![0; file.block_size as usize];

        // read block to vec
        let read_bytes = open_file
            .read(&mut buffer)
            .await
            .context("reading the block failed")?;
        buffer.truncate(read_bytes);

        Ok(Some(buffer))
    }

    pub async fn list_files(&self) -> Result<Vec<File>> {
        self.group_store.read().await.list_files()
    }

    /// Returns all files in the directory pointed to by path, from a certain user.
    /// This function only returns the files directly in that folder, and not recursively.
    pub async fn get_files_from_user(
        &self,
        user: &PublicUser,
        path: &Path,
    ) -> Result<HashSet<SimpleFile>> {
        let filetree = self.group_store.read().await.get_filetree(user)?;

        let dir = filetree.find(path).context("no file exists at this path")?;

        Ok(match dir {
            FileTree::Leaf { file, .. } => {
                let mut hs = HashSet::new();
                hs.insert(file.simplify());
                hs
            }
            FileTree::Node { children, .. } => children
                .iter()
                .map(|node| match node {
                    FileTree::Leaf { file, .. } => file.simplify(),
                    FileTree::Node { .. } => todo!(),
                })
                .collect(),
        })
    }
}

impl<S> Deref for Group<S> {
    type Target = StoredGroup;

    fn deref(&self) -> &Self::Target {
        &self.stored_group
    }
}

impl<S> DerefMut for Group<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stored_group
    }
}
