use crate::fs::group::StoredGroup;
use crate::user::PublicUser;
use anyhow::Result;
use ring::pkcs8;
use ring::signature::Ed25519KeyPair;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod heed;
pub mod inmemory;

// NOTE: This bound is not yet enforced.
#[allow(type_alias_bounds)]
pub type SharedStore<S: Store> = Arc<RwLock<Box<S>>>;

pub trait Store: Send + Sync {
    /// Creates a Arc<Rwlock<Box<dyn Store>>> :)
    fn shared(self) -> SharedStore<Self>;

    /// Saves the private user in the global_store
    fn set_self_user(&mut self, user: PublicUser) -> Result<()>;

    /// Returns the public user saved in the global_store or None if it doesn't exist
    fn get_self_user(&self) -> Result<Option<PublicUser>>;

    /// Saves the private key inside of the global_store
    fn set_signing_key(&mut self, key: pkcs8::Document) -> Result<()>;

    /// returns the ed25119 key pair based on the global_store private key
    fn get_signing_key(&self) -> Result<Option<Ed25519KeyPair>>;

    /// Add a new group to the global store.
    /// Each group should be uniquely identified by it's path, therefore it is considered
    /// an error to add a group to the store at a path which is already in the store.
    fn add_group(&mut self, group: StoredGroup) -> Result<()>;

    /// Gets the group corresponding ot its uuid
    fn get_group(&self, uuid: Uuid) -> Result<Option<StoredGroup>>;

    /// get a list of all groups
    fn get_groups(&self) -> Result<Vec<StoredGroup>>;

    /// Updates a group. The path of the group must be the same as the original group.
    /// Returns an error if the group didn't exist.
    fn update_group(&mut self, group: &StoredGroup) -> Result<()>;

    /// Removes a group from the store.
    fn delete_group(&mut self, group: &StoredGroup) -> Result<()>;
}
