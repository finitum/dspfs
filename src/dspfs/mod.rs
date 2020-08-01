use crate::global_store::{SharedStore, Store};
use crate::user::{PrivateUser, PublicUser};
use anyhow::Result;
use log::*;
use std::collections::HashMap;
use std::mem;
use std::path::Path;
use crate::fs::group::StoredGroup;
use std::fs;
use crate::dspfs::server::{Server, ServerHandle};
use crate::dspfs::client::Client;
use crate::dspfs::builder::DspfsBuilder;

pub mod builder;
pub mod server;
pub mod client;

pub struct Dspfs<S: Store + 'static> {
    pub(self) store: SharedStore<S>,
    pub(self) me: PrivateUser,
    pub(self) server: Option<Server<S>>,

    clients: HashMap<PublicUser, Client>,
    serverhandle: Option<ServerHandle>,
}

impl<S: Store> Dspfs<S> {

    pub fn builder() -> DspfsBuilder {
        DspfsBuilder::new()
    }

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
                self.server.replace(Server::new(serverhandle.addr, self.store.clone()).await?);

                serverhandle.stop().await?;
            }
        } else {
            warn!("Dspfs was already stopped, ignoring stop request");
        }

        Ok(())
    }
    
    pub async fn new_group(&mut self, path: impl AsRef<Path>) -> Result<()> {
        // 1. create or find folder (mkdir -p)
        // a)
        // 2. create .dspfs folder inside of that folder
        // 2.1: build file tree
        // 2.2: schedule index

        // b)
        // 2. Import the existing .dspfs folder

        let group = StoredGroup::new(&path);

        if group.dspfs_folder().exists() {
            // Existing folder
            todo!()
        } else {
            // New folder
            fs::create_dir_all(group.dspfs_folder())?;
            self.store.write().await.add_group(group)?;
        }

        Ok(())
    }
}

//
// #[async_trait]
// impl<S: Store> Notify for Dspfs<S> {
//     async fn file_added(&mut self, _file: &File) -> Result<()> {
//         unimplemented!()
//     }
// }
