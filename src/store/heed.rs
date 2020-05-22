use crate::fs::local::Group;
use crate::store::nested::heed::NestedHeedStore;
use crate::store::nested::NestedStore;
use crate::store::{SharedStore, Store};
use crate::user::PublicUser;
use anyhow::Result;
use heed::types::{SerdeBincode, UnalignedType};
use heed::{BytesEncode, Database, Env, EnvOpenOptions, PolyDatabase};
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
        let mut rtxn = self.env.read_txn()?;

        sell
    }

    fn set_signing_key(&mut self, key: Document) -> Result<()> {
        // Create write transaction
        let mut wtxn = self.env.write_txn()?;
        // save user under SigningKey
        self.main_db
            .put::<_, UnalignedType<PolyKey>, SerdeBincode<Document>>(
                &mut wtxn,
                &PolyKey::SigningKey,
                &key,
            )?;
        // And finally commit the changes.
        wtxn.commit()?;

        Ok(())
    }

    fn get_signing_key(&self) -> Option<Result<Ed25519KeyPair>> {
        unimplemented!()
    }

    fn get_groups(&self) -> &Vec<Group> {
        unimplemented!()
    }

    fn add_group(&mut self, group: Group) -> Result<()> {
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
