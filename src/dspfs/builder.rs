use crate::dspfs::Dspfs;
use crate::global_store::inmemory::InMemoryStore;
use crate::global_store::{SharedStore, Store};
use crate::user::PrivateUser;
use anyhow::Result;
use ring::pkcs8::Document;
use std::collections::HashMap;
use tokio::net::ToSocketAddrs;
use crate::dspfs::server::Server;

pub struct DspfsBuilder {}

impl DspfsBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn in_memory(self) -> DspfsBuilderInMemory {
        DspfsBuilderInMemory {}
    }

    // pub fn from_disk(self, dbpath: impl AsRef<Path>) -> DspfsBuilderInMemory {
    //     DspfsBuilderWithStore {
    //         global_store: Arc::new(()),
    //         me: ()
    //     }
    // }
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
            me: user,
        }
    }
}

pub struct DspfsBuilderWithStore<S: Store> {
    pub(self) store: SharedStore<S>,
    pub(self) me: PrivateUser,
}

impl<S: Store + 'static> DspfsBuilderWithStore<S> {
    pub async fn serve_on(self, addr: impl ToSocketAddrs) -> Result<DspfsBuilderWithServer<S>> {
        Ok(DspfsBuilderWithServer {
            store: self.store.clone(),
            me: self.me,
            server: Server::new(addr, self.store).await?,
        })
    }
}

pub struct DspfsBuilderWithServer<S: Store + 'static> {
    pub(self) store: SharedStore<S>,
    pub(self) me: PrivateUser,
    pub(self) server: Server<S>,
}

impl<S: Store + 'static> DspfsBuilderWithServer<S> {
    pub async fn build(self) -> Dspfs<S> {
        Dspfs {
            store: self.store,
            me: self.me,
            server: Some(self.server),

            clients: HashMap::new(),
            serverhandle: None,
        }
    }
}
