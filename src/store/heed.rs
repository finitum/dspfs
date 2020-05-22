use crate::fs::local::Group;
use crate::store::nested::heed::NestedHeedStore;
use crate::store::nested::NestedStore;
use crate::store::{SharedStore, Store};
use crate::user::PublicUser;
use anyhow::{Context, Result};
use heed::types::{SerdeBincode, UnalignedSlice, UnalignedType};
use heed::{Env, EnvOpenOptions, PolyDatabase};
use ring::pkcs8::Document;
use ring::signature::Ed25519KeyPair;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use zerocopy::AsBytes;
use zerocopy::Unaligned;

pub struct HeedStore {
    db_path: PathBuf,
    env: Env,
    main_db: PolyDatabase,
}

impl HeedStore {
    pub fn new_or_load(path: PathBuf) -> Result<Self> {
        if path.extension() != Some(OsStr::new(".mdb")) {
            return Err(anyhow::anyhow!("Invalid db file extension (must be .mdb)"));
        }

        fs::create_dir_all(&path)?;
        let env = EnvOpenOptions::new().open(&path)?;

        let main_db = env.create_poly_database(None)?;

        Ok(Self {
            db_path: path,
            env,
            main_db,
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
    fn set_me(&mut self, user: PublicUser) -> Result<()> {
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

    fn get_me(&self) -> Result<Option<PublicUser>> {
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

    fn get_groups(&self) -> &Vec<Group> {
        unimplemented!()
    }

    fn add_group(&mut self, _group: Group) -> Result<()> {
        unimplemented!()
    }

    fn shared(self) -> SharedStore<Self> {
        unimplemented!()
    }

    fn create_nested_kv_store<
        K: 'static + Serialize,
        V: 'static + Serialize + for<'de> Deserialize<'de>,
    >(
        &mut self,
        name: &str,
    ) -> Result<Box<dyn NestedStore<K, V, Self>>> {
        let db = self
            .env
            .create_database::<heed::types::SerdeBincode<K>, heed::types::SerdeBincode<V>>(Some(
                name,
            ))?;

        Ok(Box::new(NestedHeedStore::new(db, self.env.clone())))
    }
}
