use crate::store::heed::HeedStore;
use crate::store::nested::NestedStore;
use anyhow::Result;
use heed::Database;
use serde::export::PhantomData;
use serde::{Deserialize, Serialize};

pub struct NestedHeedStore<K, V> {
    key_type: PhantomData<K>,
    value_type: PhantomData<V>,
    db: Database<heed::types::SerdeBincode<K>, heed::types::SerdeBincode<V>>,
    env: heed::Env,
}

impl<K, V> NestedHeedStore<K, V> {
    pub fn new(
        db: Database<heed::types::SerdeBincode<K>, heed::types::SerdeBincode<V>>,
        env: heed::Env,
    ) -> Self {
        Self {
            db,
            env,
            key_type: Default::default(),
            value_type: Default::default(),
        }
    }
}

impl<K: 'static + Serialize, V: 'static + Serialize + for<'de> Deserialize<'de>>
    NestedStore<K, V, HeedStore> for NestedHeedStore<K, V>
{
    fn get(&self, key: K) -> Result<Option<V>> {
        let rtxn = self.env.read_txn()?;
        let value = self.db.get(&rtxn, &key)?;

        let res = Ok(value);

        res
    }

    fn insert(&mut self, key: K, value: V) -> Result<()> {
        let mut wtxn = self.env.write_txn()?;
        self.db.put(&mut wtxn, &key, &value)?;
        wtxn.commit()?;

        Ok(())
    }
}
