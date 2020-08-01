#![allow(dead_code)]

use log::*;
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

    let _store = global_store::inmemory::InMemoryStore::default();

    // Run program
    info!("Hello world!");

    Ok(())
}
