use crate::fs::group::StoredGroup;
use crate::global_store::{SharedStore, Store};
use crate::user::PublicUser;
use anyhow::{Context, Result};
use ring::pkcs8::Document;
use ring::signature::Ed25519KeyPair;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InMemoryStore {
    user: Option<PublicUser>,
    signing_key: Option<Document>,
    groups: HashMap<Uuid, StoredGroup>,
}

impl Default for InMemoryStore {
    fn default() -> InMemoryStore {
        Self {
            user: None,
            signing_key: None,
            groups: HashMap::new(),
        }
    }
}

impl Store for InMemoryStore {
    fn set_self_user(&mut self, user: PublicUser) -> Result<()> {
        self.user = Some(user);
        Ok(())
    }

    fn get_self_user(&self) -> Result<Option<PublicUser>> {
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

    fn add_group(&mut self, group: StoredGroup) -> Result<()> {
        if let Entry::Vacant(e) = self.groups.entry(group.uuid) {
            e.insert(group);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Can't add a group at a location in the filesystem which already has a group."
            ))
        }
    }

    fn get_groups(&self) -> Result<Vec<StoredGroup>> {
        Ok(self.groups.values().cloned().collect())
    }

    fn update_group(&mut self, group: &StoredGroup) -> Result<()> {
        if let Entry::Occupied(mut e) = self.groups.entry(group.uuid) {
            *e.get_mut() = group.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Can't update a group which doesn't exist yet"
            ))
        }
    }

    fn delete_group(&mut self, group: &StoredGroup) -> Result<()> {
        self.groups
            .remove(&group.uuid)
            .context("Can't remove a group which is not (yet) in the store")?;
        Ok(())
    }

    fn shared(self) -> SharedStore<Self> {
        Arc::new(RwLock::new(Box::new(self)))
    }

    fn get_group(&self, uuid: Uuid) -> Result<Option<StoredGroup>> {
        Ok(self.groups.get(&uuid).cloned())
    }
}

#[cfg(test)]
use crate::user::PrivateUser;
use uuid::Uuid;

#[cfg(test)]
impl InMemoryStore {
    pub fn test_store(username: &str) -> Result<SharedStore<InMemoryStore>> {
        let mut res = Self::default();
        let (pu, doc) = PrivateUser::new(username)?;
        res.set_self_user(pu.public_user().to_owned()).unwrap();
        res.set_signing_key(doc).unwrap();

        Ok(res.shared())
    }
}
