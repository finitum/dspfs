use log::*;
use dotenv;

mod api;

pub fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().unwrap();

    // Init program
    pretty_env_logger::init();

    // Run program
    info!("Hello world!");
}
