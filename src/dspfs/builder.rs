use crate::dspfs::server::Server;
use crate::dspfs::Dspfs;
use crate::global_store::inmemory::InMemoryStore;
use crate::global_store::{SharedStore, Store};
use crate::user::PrivateUser;
use anyhow::Result;
use ring::pkcs8::Document;
use std::collections::HashMap;
use std::path::Path;
use tokio::net::ToSocketAddrs;

// TODO: Rethink

pub struct DspfsBuilder;

impl DspfsBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn in_memory(self) -> DspfsBuilderInMemory {
        DspfsBuilderInMemory {}
    }

    pub fn with_store<S: Store>(self, store: SharedStore<S>) -> DspfsBuilderWithStore<S> {
        DspfsBuilderWithStore { store }
    }
}

pub struct DspfsBuilderInMemory {}

impl DspfsBuilderInMemory {
    pub fn with_user(
        &self,
        (user, document): (PrivateUser, Document),
    ) -> DspfsBuilderWithStore<InMemoryStore> {
        let mut store = InMemoryStore::default();
        store.set_signing_key(document).unwrap();
        store.set_self_user(user.public_user().to_owned()).unwrap();

        DspfsBuilderWithStore {
            store: store.shared(),
        }
    }
}

pub struct DspfsBuilderWithStore<S: Store> {
    pub(self) store: SharedStore<S>,
}

impl<S: Store + 'static> DspfsBuilderWithStore<S> {
    pub async fn serve_on(self, addr: impl ToSocketAddrs) -> Result<DspfsBuilderWithServer<S>> {
        Ok(DspfsBuilderWithServer {
            store: self.store.clone(),
            server: Server::new(addr, self.store).await?,
        })
    }
}

pub struct DspfsBuilderWithServer<S: Store + 'static> {
    pub(self) store: SharedStore<S>,
    pub(self) server: Server<S>,
}

impl<S: Store + 'static> DspfsBuilderWithServer<S> {
    pub async fn build(self) -> Dspfs<S> {
        Dspfs {
            store: self.store,
            server: Some(self.server),

            clients: HashMap::new(),
            serverhandle: None,
        }
    }
}
