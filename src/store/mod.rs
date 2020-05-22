use crate::fs::local::Group;
use crate::store::nested::NestedStore;
use crate::user::PublicUser;
use anyhow::Result;
use ring::pkcs8;
use ring::signature::Ed25519KeyPair;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod heed;
pub mod inmemory;
pub mod nested;

// TODO: S: Store (this bound is enforced in the next release of rust)
pub type SharedStore<S> = Arc<RwLock<Box<S>>>;

pub trait Store: Send + Sync {
    /// Saves the private user in the store
    fn set_me(&mut self, user: PublicUser) -> Result<()>;

    /// Returns the private user saved in the store or None if it doesn't exist
    fn get_me(&self) -> Result<Option<PublicUser>>;

    /// Saves the private key inside of the store
    fn set_signing_key(&mut self, key: pkcs8::Document) -> Result<()>;

    ///returns the ed25119 key pair based on the store private key
    fn get_signing_key(&self) -> Result<Option<Ed25519KeyPair>>;

    /// Gets a list of groups in the store
    fn get_groups(&self) -> &Vec<Group>;
    /// Add a group to the storeb
    fn add_group(&mut self, group: Group) -> Result<()>;

    /// Creates a Arc<Rwlock<Box<dyn Store>>> :)
    fn shared(self) -> SharedStore<Self>;

    fn create_nested_kv_store<
        K: 'static + serde::Serialize,
        V: 'static + serde::Serialize + for<'de> serde::Deserialize<'de> + Clone,
    >(
        &mut self,
        name: &str,
    ) -> Result<Box<dyn NestedStore<K, V, Self>>>;
}
