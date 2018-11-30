use data_manager::DataManager;
use datatypes::Message;
use datatypes::MessageSendRequest;
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
        let req: String = match msg.into_text() {
            Ok(req) => req,
            Err(_) => {
                println!("Failed to decode message as text.");
                return Ok(());
            }
        };

        let req: SocketMessage = match serde_json::from_str(&req) {
            Ok(req) => req,
            Err(_) => {
                println!("Failed to parse request '{}'", req);
                return Ok(());
            }
        };

        match req.method {
            MethodName::Login => {
                let username = req.data;
                let mut um = self.dm.lock().unwrap();
                let login_result = match um.get_or_create(&username) {
                    Some(user) => {
                        let lr = um.generate_login_result(&user);
                        self.token = Some(lr.token.clone());

                        self.sender.send(
                            serde_json::to_string(&SocketMessage {
                                method: MethodName::Messages,
                                data: serde_json::to_string(&um.generate_messages_result_all())
                                    .unwrap(),
                            }).unwrap(),
                        )?;

                        Some(lr)
                    }
                    None => None,
                };
                let resp = SocketMessage {
                    method: MethodName::Login,
                    data: serde_json::to_string(&login_result).unwrap(),
                };
                self.sender.send(serde_json::to_string(&resp).unwrap())?
            }
            MethodName::Messages => {
                match &self.token {
                    None => return Ok(()),
                    Some(token) => {
                        let mut um = self.dm.lock().unwrap();
                        let resp = SocketMessage {
                            method: MethodName::Messages,
                            data: serde_json::to_string(&um.add_message(req.data, &token)).unwrap(),
                        };
                        self.sender.send(serde_json::to_string(&resp).unwrap())?
                    }
                };
            }
            _ => {}
        }
        Ok(())
    }
}
