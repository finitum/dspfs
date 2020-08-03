#![allow(dead_code)]

use anyhow::Context;
use std::env;
use std::error::Error;

mod dspfs;
mod fs;
mod global_store;
mod message;
mod stream;
mod user;

fn init() {
    // Load environment variables from .env file
    dotenv::dotenv().unwrap();

    // Init program
    let _ = pretty_env_logger::try_init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init();

    let config_dir = env::var("DSPFS_CONFIG_DIR")
        .map(|i| i.into())
        .or_else::<anyhow::Error, _>(|_|{
            let mut dir = dirs::config_dir().context("Couldn't find default configuration directory")?;
            dir.push("dspfs");
            Ok(dir)
        }).expect("Couldn't find default config location and no user override given (envvar: DSPFS_CONFIG_DIR)");

    log::info!("Using {:?} as configuration directory", config_dir);

    let mut store_location = config_dir;
    store_location.push("dspfs.mdb");

    let _store = global_store::heed::HeedStore::new_or_load(store_location)
        .context("Couldn't create or connect to database")?;

    // Run program

    
    
    Ok(())
}
