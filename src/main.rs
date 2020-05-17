#![allow(dead_code)]

use log::*;
use std::error::Error;

mod stream;
mod error;
mod message;
mod store;
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

    let _store = store::inmemory::InMemory::default();

    // Run program
    info!("Hello world!");

    Ok(())
}
