use crate::dspfs::notify::Notify;
use crate::fs::shared::File;
use crate::store::{SharedStore, Store};
use crate::stream::{Client, Server, ServerHandle};
use crate::user::{PrivateUser, PublicUser};
use anyhow::Result;
use async_trait::async_trait;
use log::*;
use std::collections::HashMap;
use std::mem;

pub mod builder;
pub mod notify;

pub struct Dspfs<S: Store + 'static> {
    pub(self) store: SharedStore<S>,
    pub(self) me: PrivateUser,
    pub(self) server: Option<Server<S>>,

    clients: HashMap<PublicUser, Client>,
    serverhandle: Option<ServerHandle>,
}

impl<S: Store> Dspfs<S> {
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
                mem::replace(
                    &mut self.server,
                    Some(Server::new(serverhandle.addr, self.store.clone()).await?),
                );
                serverhandle.stop().await?;
            }
        } else {
            warn!("Dspfs was already stopped, ignoring stop request");
        }

        Ok(())
    }
}

#[async_trait]
impl<S: Store> Notify for Dspfs<S> {
    async fn file_added(&mut self, _file: &File) -> Result<()> {
        unimplemented!()
    }
}
