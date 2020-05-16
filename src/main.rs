#![allow(dead_code)]

use log::*;
use std::error::Error;

mod node;
mod protos;
mod user;

fn init() {
    // Load environment variables from .env file
    dotenv::dotenv().unwrap();

    // Init program
    pretty_env_logger::init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init();

    // Run program
    info!("Hello world!");

    Ok(())
}
