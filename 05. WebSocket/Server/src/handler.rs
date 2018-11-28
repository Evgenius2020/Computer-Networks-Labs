use data_manager::DataManager;
use datatypes::{MethodName, SocketMessage};
use serde_json;
use std::sync::{Arc, Mutex};
use ws;

pub struct Handler {
    sender: ws::Sender,
    token: Option<String>,
    dm: Arc<Mutex<DataManager>>,
}

impl Handler {
    pub fn new(sender: ws::Sender, dm: Arc<Mutex<DataManager>>) -> Handler {
        Handler {
            sender: sender,
            token: None,
            dm: dm,
        }
    }
}

impl ws::Handler for Handler {
    fn on_message(&mut self, msg: ws::Message) -> Result<(), ws::Error> {
        let req = msg.into_text()?;
        let req: SocketMessage = match serde_json::from_str(&req) {
            Ok(message) => message,
            Err(_) => {
                return Err(ws::Error::new(
                    ws::ErrorKind::Internal,
                    "Failed to parse request",
                ))
            }
        };
        match req.method {
            MethodName::Login => {
                let username = req.data;
                let user = (*self.dm.lock().unwrap()).get_or_create(&username);
                self.sender.send(serde_json::to_string(&user).unwrap())?
            }
            _ => {}
        }

        Ok(())
    }
}
