use crate::error::DspfsError;
use crate::fs::shared;
use crate::store::SharedStore;
use crate::user::PublicUser;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub struct Group {
    files: HashMap<PublicUser, HashMap<PathBuf, shared::File>>,
    users: HashSet<PublicUser>,
}

impl Group {
    /// Adds a file to the hashmap of a user. This can be `me`.
    /// TODO: when a file is added, we should probably broadcast that this new file
    ///       exists so others can download it (That's why this fn is async)
    pub async fn add_file(&self, _from: &PublicUser, _file: shared::File) {}

    pub async fn get_my_files(
        &self,
        store: SharedStore,
    ) -> Result<impl Iterator<Item = &shared::File>, DspfsError> {
        let guard = store.read().await;
        let me = guard.get_self_user().as_ref().ok_or_else(|| {
            DspfsError::NotFoundInStore("Group::add_file(): Could not find user in store".into())
        })?;

        Ok(self
            .files
            .get(&me)
            .ok_or_else(|| DspfsError::from("User not found"))?
            .values())
    }
}
