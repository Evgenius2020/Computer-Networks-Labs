extern crate uuid;
#[macro_use]
extern crate serde_derive;
extern crate env_logger;
extern crate serde_json;
extern crate ws;

mod data_manager;
mod datatypes;
mod handler;

use data_manager::DataManager;
use handler::Handler;
use std::sync::{Arc, Mutex};

const FILENAME: &str = "target/db.json";

fn main() {
    env_logger::init();

    let dm = match DataManager::load(FILENAME) {
        Some(dm) => dm,
        None => DataManager::new(FILENAME.to_string()),
    };
    let dm = Arc::new(Mutex::new(dm));

    if let Err(error) = ws::listen("127.0.0.1:1337", |out| Handler::new(out, dm.clone())) {
        println!("Failed to create WebSocket due to {:?}", error);
    }
}
