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
    fn on_close(&mut self, _code: ws::CloseCode, _reason: &str) {
        match &self.token {
            None => {}
            Some(token) => {
                let mut dm = self.dm.lock().unwrap();
                match dm.get_user_id_by_token(&token) {
                    None => {}
                    Some(user_id) => {
                        &dm.logout(&token);
                        let user = dm.get_user(user_id);
                        let resp = SocketMessage {
                            method: MethodName::Users,
                            data: serde_json::to_string(&user).unwrap(),
                        };
                        self.sender
                            .broadcast(serde_json::to_string(&resp).unwrap())
                            .unwrap();
                    }
                }
            }
        }
    }

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
                let mut dm = self.dm.lock().unwrap();
                let login_result = match dm.get_or_create_user(&username) {
                    Some(user) => {
                        let lr = dm.login(&user);
                        self.token = Some(lr.token.clone());

                        self.sender.send(
                            serde_json::to_string(&SocketMessage {
                                method: MethodName::Messages,
                                data: serde_json::to_string(&dm.get_messages()).unwrap(),
                            }).unwrap(),
                        )?;

                        self.sender.send(
                            serde_json::to_string(&SocketMessage {
                                method: MethodName::Users,
                                data: serde_json::to_string(&dm.get_users()).unwrap(),
                            }).unwrap(),
                        )?;

                        let user = dm.get_user(lr.id);
                        let resp = SocketMessage {
                            method: MethodName::Users,
                            data: serde_json::to_string(&user).unwrap(),
                        };
                        self.sender
                            .broadcast(serde_json::to_string(&resp).unwrap())
                            .unwrap();

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
                        let mut dm = self.dm.lock().unwrap();
                        let resp = SocketMessage {
                            method: MethodName::Messages,
                            data: serde_json::to_string(&dm.add_message(req.data, &token)).unwrap(),
                        };
                        self.sender
                            .broadcast(serde_json::to_string(&resp).unwrap())?
                    }
                };
            }
            MethodName::Logout => {
                match &self.token {
                    None => return Ok(()),
                    Some(_) => {
                        self.sender.close(ws::CloseCode::Normal)?
                    }
                };
            }
            _ => {}
        }
        Ok(())
    }
}
