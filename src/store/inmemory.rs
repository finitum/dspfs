use crate::fs::local::Group;
use crate::store::nested::inmemory::NestedInMemoryStore;
use crate::store::nested::NestedStore;
use crate::store::{SharedStore, Store};
use crate::user::PublicUser;
use anyhow::Result;
use ring::pkcs8::Document;
use ring::signature::Ed25519KeyPair;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InMemoryStore {
    user: Option<PublicUser>,
    signing_key: Option<Document>,

    groups: Vec<Group>,
}

impl Default for InMemoryStore {
    fn default() -> InMemoryStore {
        Self {
            user: None,
            signing_key: None,
            groups: vec![],
        }
    }
}

impl Store for InMemoryStore {
    fn set_me(&mut self, user: PublicUser) -> Result<()> {
        self.user = Some(user);
        Ok(())
    }

    fn get_me(&self) -> Result<Option<PublicUser>> {
        Ok(self.user.clone())
    }

    fn set_signing_key(&mut self, key: Document) -> Result<()> {
        self.signing_key = Some(key);
        Ok(())
    }

    fn get_signing_key(&self) -> Result<Option<Ed25519KeyPair>> {
        if let Some(ref bytes) = self.signing_key {
            Ok(Some(
                Ed25519KeyPair::from_pkcs8(bytes.as_ref())
                    .map_err(|_| anyhow::anyhow!("key rejected"))?,
            ))
        } else {
            Ok(None)
        }
    }

    fn get_groups(&self) -> &Vec<Group> {
        &self.groups
    }

    fn add_group(&mut self, group: Group) -> Result<()> {
        self.groups.push(group);
        Ok(())
    }

    fn shared(self) -> SharedStore<Self> {
        Arc::new(RwLock::new(Box::new(self)))
    }

    fn create_nested_kv_store<
        K: 'static + Serialize,
        V: 'static + Serialize + for<'de> Deserialize<'de> + Clone,
    >(
        &mut self,
        _: &str,
    ) -> Result<Box<dyn NestedStore<K, V, Self>>> {
        Ok(Box::new(NestedInMemoryStore::new()))
    }
}

#[cfg(test)]
use crate::user::PrivateUser;

#[cfg(test)]
impl InMemoryStore {
    pub fn test_store(username: &str) -> Result<SharedStore<InMemoryStore>> {
        let mut res = Self::default();
        let (pu, doc) = PrivateUser::new(username)?;
        res.set_me(pu.public_user().to_owned()).unwrap();
        res.set_signing_key(doc).unwrap();

        Ok(res.shared())
    }
}
