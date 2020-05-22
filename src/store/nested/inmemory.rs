use crate::store::inmemory::InMemoryStore;
use crate::store::nested::NestedStore;
use anyhow::{Context, Result};
use serde::ser::Serialize;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct NestedInMemoryStore<K: Serialize + 'static, V> {
    store: HashMap<Vec<u8>, V>,

    phantom: PhantomData<K>,
}

impl<K: Serialize, V> Default for NestedInMemoryStore<K, V> {
    fn default() -> Self {
        Self {
            store: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<K: Serialize, V> NestedInMemoryStore<K, V> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<K: Serialize + 'static, V: 'static + Clone> NestedStore<K, V, InMemoryStore>
    for NestedInMemoryStore<K, V>
{
    fn get(&self, key: K) -> Result<Option<V>> {
        let sk = bincode::serialize(&key).context("failed serializing")?;

        let res = self.store.get(&sk).cloned();

        Ok(res)
    }

    fn insert(&mut self, key: K, value: V) -> Result<()> {
        let sk = bincode::serialize(&key).context("failed serializing")?;

        self.store.insert(sk, value);

        Ok(())
    }
}
