use crate::store::Store;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod heed;
pub mod inmemory;

// TODO: S: NestedStore (this bound is enforced in the next release of rust)
pub type SharedNestedStore<S> = Arc<RwLock<Box<S>>>;

pub trait NestedStore<K: 'static, V: 'static, S: Store> {
    fn get(&self, key: K) -> Result<Option<V>>;
    fn insert(&mut self, key: K, value: V) -> Result<()>;
}
