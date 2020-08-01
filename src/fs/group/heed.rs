use crate::fs::file::File;
use crate::fs::filetree::FileTree;
use crate::fs::group::store::GroupStore;
use crate::fs::hash::Hash;
use crate::user::PublicUser;
use anyhow::{Context, Result};
use heed::types::SerdeBincode;
use heed::{Database, Env, EnvOpenOptions};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub struct HeedGroupStore {
    env: Env,
    filetrees: Database<SerdeBincode<PublicUser>, SerdeBincode<FileTree>>,
    files: Database<SerdeBincode<Hash>, SerdeBincode<File>>,
}

impl HeedGroupStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        if path.as_ref().extension() != Some(OsStr::new("mdb")) {
            return Err(anyhow::anyhow!("Invalid db file extension (must be .mdb)"));
        }

        fs::create_dir_all(&path)?;
        let mut opts = EnvOpenOptions::new();
        opts.max_dbs(2);
        let env = opts.open(&path)?;

        let filetrees = env.create_database(Some("filetrees"))?;
        let files = env.create_database(Some("files"))?;

        Ok(Self {
            env,
            filetrees,
            files,
        })
    }
}

impl GroupStore for HeedGroupStore {
    fn add_file(&mut self, user: &PublicUser, file: File) -> Result<()> {
        let mut wtxn = self.env.write_txn()?;

        self.files.put(&mut wtxn, &file.hash, &file)?;

        let mut tree = self
            .filetrees
            .get(&wtxn, user)
            .context("Error accessing the db")?
            .unwrap_or_else(FileTree::new);

        tree.insert(file.path.clone(), file)?;

        self.filetrees
            .put(&mut wtxn, user, &tree)
            .context("error saving to the db")?;

        wtxn.commit()?;

        Ok(())
    }

    fn get_file(&self, hash: Hash) -> Result<Option<File>> {
        let rtxn = self.env.read_txn()?;

        Ok(self
            .files
            .get(&rtxn, &hash)
            .context("error getting hash from db")?)
    }

    fn delete_file(&mut self, user: &PublicUser, file: &File) -> Result<()> {
        let mut wtxn = self.env.write_txn()?;

        if let Some(mut tree) = self
            .filetrees
            .get(&wtxn, user)
            .context("Error accessing the db")?
        {
            tree.delete(&file.path, false);

            self.filetrees
                .put(&mut wtxn, user, &tree)
                .context("error saving to the db")?;
        }

        if let Some(mut file) = self
            .files
            .get(&wtxn, &file.hash)
            .context("Error accessing the db")?
        {
            file.remove_user(user);
            if file.num_owning_users() > 0 {
                self.files.put(&mut wtxn, &file.hash, &file)?;
            } else {
                self.files.delete(&mut wtxn, &file.hash)?;
            }
        }

        wtxn.commit()?;
        Ok(())
    }
}
