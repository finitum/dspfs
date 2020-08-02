use crate::fs::group::StoredGroup;
use crate::global_store::{SharedStore, Store};
use crate::user::PublicUser;
use anyhow::{Context, Result};
use heed::types::{SerdeBincode, UnalignedSlice, UnalignedType};
use heed::{Database, Env, EnvOpenOptions, PolyDatabase};
use ring::pkcs8::Document;
use ring::signature::Ed25519KeyPair;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zerocopy::AsBytes;
use zerocopy::Unaligned;

pub struct HeedStore {
    db_path: PathBuf,
    env: Env,
    main_db: PolyDatabase,
    groups_db: Database<SerdeBincode<Uuid>, SerdeBincode<StoredGroup>>,
}

impl HeedStore {
    pub fn new_or_load(path: impl AsRef<Path>) -> Result<Self> {
        if path.as_ref().extension() != Some(OsStr::new("mdb")) {
            return Err(anyhow::anyhow!("Invalid db file extension (must be .mdb)"));
        }

        fs::create_dir_all(&path)?;
        let mut env_opts = EnvOpenOptions::new();
        env_opts.max_dbs(2);
        let env = env_opts.open(&path)?;

        let main_db = env.create_poly_database(None)?;
        let groups_db = env.create_database(Some("groups"))?;

        Ok(Self {
            db_path: path.as_ref().to_path_buf(),
            env,
            main_db,
            groups_db,
        })
    }
}

#[derive(AsBytes, Unaligned, Debug)]
#[repr(u8)]
enum PolyKey {
    Me,
    SigningKey,
}

impl Store for HeedStore {
    fn shared(self) -> SharedStore<Self> {
        unimplemented!()
    }

    fn set_self_user(&mut self, user: PublicUser) -> Result<()> {
        // Create write transaction
        let mut wtxn = self.env.write_txn()?;
        // save user under SELF_USER_KEY
        self.main_db
            .put::<_, UnalignedType<PolyKey>, SerdeBincode<PublicUser>>(
                &mut wtxn,
                &PolyKey::Me,
                &user,
            )?;
        // And finally commit the changes.
        wtxn.commit()?;

        Ok(())
    }

    fn get_self_user(&self) -> Result<Option<PublicUser>> {
        let rtxn = self.env.read_txn()?;

        let me: Option<PublicUser> = self
            .main_db
            .get::<_, UnalignedType<PolyKey>, SerdeBincode<PublicUser>>(&rtxn, &PolyKey::Me)
            .context("Accessing the DB went wrong")?;

        Ok(me)
    }

    fn set_signing_key(&mut self, key: Document) -> Result<()> {
        // Create write transaction
        let mut wtxn = self.env.write_txn()?;

        let doc_bytes = key.as_ref();
        // save user under SigningKey
        self.main_db
            .put::<_, UnalignedType<PolyKey>, UnalignedSlice<u8>>(
                &mut wtxn,
                &PolyKey::SigningKey,
                &doc_bytes,
            )?;
        // And finally commit the changes.
        wtxn.commit()?;

        Ok(())
    }

    fn get_signing_key(&self) -> Result<Option<Ed25519KeyPair>> {
        let rtxn = self.env.read_txn()?;

        let signing_key = self
            .main_db
            .get::<_, UnalignedType<PolyKey>, UnalignedSlice<u8>>(&rtxn, &PolyKey::SigningKey)
            .context("Accessing the DB went wrong")?;

        if let Some(bytes) = signing_key {
            Ok(Some(
                Ed25519KeyPair::from_pkcs8(bytes).map_err(|_| anyhow::anyhow!("key rejected"))?,
            ))
        } else {
            Ok(None)
        }
    }

    fn add_group(&mut self, group: StoredGroup) -> Result<()> {
        // Create write transaction
        let mut wtxn = self.env.write_txn()?;

        // See if the group already existed
        if self.groups_db.get(&wtxn, &group.uuid)?.is_some() {
            return Err(anyhow::anyhow!(
                "Can't create a group at a location in the filesystem which already has a group."
            ));
        }

        // save the group
        self.groups_db.put(&mut wtxn, &group.uuid, &group)?;
        // And finally commit the changes.
        wtxn.commit()?;

        Ok(())
    }

    fn get_group(&self, uuid: Uuid) -> Result<Option<StoredGroup>> {
        let rtxn = self.env.read_txn()?;

        let res = self.groups_db.get(&rtxn, &uuid)?;

        Ok(res)
    }

    fn get_groups(&self) -> Result<Vec<StoredGroup>> {
        let rtxn = self.env.read_txn()?;

        let res = self
            .groups_db
            .iter(&rtxn)?
            .filter_map(|i| {
                if let Ok((_path, group)) = i {
                    Some(group)
                } else {
                    None
                }
            })
            .collect();

        Ok(res)
    }

    fn update_group(&mut self, group: &StoredGroup) -> Result<()> {
        // Create write transaction
        let mut wtxn = self.env.write_txn()?;

        if self.groups_db.get(&wtxn, &group.uuid)?.is_some() {
            self.groups_db.put(&mut wtxn, &group.uuid, group)?;
            wtxn.commit()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Can't update a group which doesn't exist yet"
            ))
        }
    }

    fn delete_group(&mut self, group: &StoredGroup) -> Result<()> {
        // Create write transaction
        let mut wtxn = self.env.write_txn()?;

        self.groups_db.delete(&mut wtxn, &group.uuid)?;

        wtxn.commit()?;

        Ok(())
    }
}
