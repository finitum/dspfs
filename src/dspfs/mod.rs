use crate::dspfs::builder::DspfsBuilder;
use crate::dspfs::client::Client;
use crate::dspfs::server::{Server, ServerHandle};
use crate::fs::file::File;
use crate::fs::file::SimpleFile;
use crate::fs::group::StoredGroup;
use crate::fs::hash::Hash;
use crate::global_store::{SharedStore, Store};
use crate::user::{PrivateUser, PublicUser};
use anyhow::{Context, Result};
use api::Api;
use api::LocalFile;
use async_trait::async_trait;
use log::*;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::mem;
use std::path::Path;
use uuid::Uuid;

pub mod api;
pub mod builder;
pub mod client;
pub mod server;

pub struct Dspfs<S> {
    pub(self) store: SharedStore<S>,
    pub(self) server: Option<Server<S>>,

    clients: HashMap<PublicUser, Client>,
    serverhandle: Option<ServerHandle>,
}


impl<S: Store + 'static> Dspfs<S> {
    pub async fn start(&mut self) {
        if self.server.is_some() {
            if let Some(server) = mem::replace(&mut self.server, None) {
                server.start().await;
            }
        } else {
            warn!("Dspfs was already started, ignoring start request");
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        if self.serverhandle.is_some() {
            if let Some(serverhandle) = mem::replace(&mut self.serverhandle, None) {
                self.server
                    .replace(Server::new(serverhandle.addr, self.store.clone()).await?);

                serverhandle.stop().await?;
            }
        } else {
            warn!("Dspfs was already stopped, ignoring stop request");
        }

        Ok(())
    }
}

#[async_trait]
impl<S: Store> Api for Dspfs<S> {
    async fn init_group(&self, path: &Path) -> Result<Uuid> {
        let group = StoredGroup::new(&path);

        if !group.dspfs_folder().exists() {
            // New folder
            fs::create_dir_all(group.dspfs_folder())?;
            let uuid = group.uuid;
            self.store.write().await.add_group(group)?;

            Ok(uuid)
        } else {
            Err(anyhow::anyhow!("Group already exists"))
        }
    }

    async fn add_file(&self, guuid: Uuid, path: &Path) -> Result<SimpleFile> {
        let mut group = self
            .store
            .read()
            .await
            .get_group(guuid)?
            .context("There is no group with this uuid.")?
            .reload(self.store.clone())?;

        let us = self.store.read().await.get_self_user()?.context("")?;

        let mut location = group.dspfs_root().to_path_buf();
        location.push(path);

        if !location.is_file() {
            return Err(anyhow::anyhow!("Path does not point to a file"));
        }

        let file = File::new(location).await.context("Indexing file failed")?;

        let simple_file = file.simplify();

        group
            .add_file(&us, file)
            .await
            .context("Adding file to group failed")?;

        Ok(simple_file)
    }

    async fn join_group(&self, _guuid: Uuid, _bootstrap_user: &PublicUser) -> Result<()> {
        unimplemented!("procrastination is life")
    }

    async fn list_files(&self, guuid: Uuid) -> Result<HashSet<SimpleFile>> {
        let group = self
            .store
            .read()
            .await
            .get_group(guuid)?
            .context("There is no group with this uuid.")?
            .reload(self.store.clone())?;

        let f = group.list_files().await?.into_iter().map(|f| f.simplify());

        Ok(f.collect())
    }

    async fn get_users(&self, guuid: Uuid) -> Result<BTreeSet<PublicUser>> {
        let group = self
            .store
            .read()
            .await
            .get_group(guuid)?
            .context("There is no group with this uuid.")?;

        Ok(group.users)
    }

    async fn get_available_files(&self, guuid: Uuid, path: &Path) -> Result<HashSet<LocalFile>> {
        let group = self
            .store
            .read()
            .await
            .get_group(guuid)?
            .context("There is no group with this uuid.")?;

        let mut location = group.dspfs_root().to_path_buf();
        location.push(path);

        let set = location
            .read_dir()
            .context("Reading directory failed")?
            .map(|i| i.map(LocalFile::from_direntry))
            .flatten()
            .collect::<Result<_>>()?;

        Ok(set)
    }

    async fn get_files(
        &self,
        guuid: Uuid,
        user: &PublicUser,
        path: &Path,
    ) -> Result<HashSet<SimpleFile>> {
        let group = self
            .store
            .read()
            .await
            .get_group(guuid)?
            .context("There is no group with this uuid.")?
            .reload(self.store.clone())?;

        let _files = group.get_files_from_user(user, path).await;

        todo!()
    }

    async fn request_file(&self, _hash: Hash, _to: &Path) -> Result<()> {
        unimplemented!()
    }

    async fn refresh(&self) {
        unimplemented!()
    }
}
