use log::*;
use dotenv;
use std::error::Error;

mod protos;
mod node;
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
