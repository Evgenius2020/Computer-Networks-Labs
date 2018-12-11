use data_manager::DataManager;
use datatypes::{MethodName, SocketMessage};
use serde_json;
use std::sync::{Arc, Mutex};
use ws;

pub struct Handler {
    sender: ws::Sender,
    user_id: Option<usize>,
    dm: Arc<Mutex<DataManager>>,
}

impl Handler {
    pub fn new(sender: ws::Sender, dm: Arc<Mutex<DataManager>>) -> Handler {
        Handler {
            sender: sender,
            user_id: None,
            dm: dm,
        }
    }

    fn change_online_and_broadcast(&mut self, online: Option<bool>) {
        match self.user_id {
            None => {}
            Some(user_id) => {
                let mut dm = self.dm.lock().unwrap();
                &dm.delete_token(user_id);
                let users = dm.change_online(user_id, online);
                self.sender
                    .broadcast(
                        serde_json::to_string(&SocketMessage {
                            method: MethodName::Users,
                            data: serde_json::to_string(&users).unwrap(),
                        }).unwrap(),
                    ).unwrap();
            }
        }
    }
}

impl ws::Handler for Handler {
    fn on_close(&mut self, _code: ws::CloseCode, _reason: &str) {
        self.change_online_and_broadcast(None);
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
            MethodName::NameLogin => {
                let username = req.data;
                let res = 
                {
                    let mut dm = self.dm.lock().unwrap();
                    let res = match dm.name_login(&username) {
                        None => Ok(()),
                        Some(login_result) => {
                            self.user_id = Some(login_result.id);

                            let resp = SocketMessage {
                                method: MethodName::NameLogin,
                                data: serde_json::to_string(&login_result).unwrap(),
                            };
                            self.sender.send(serde_json::to_string(&resp).unwrap())
                        }
                    };

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

                    res
                };
                self.change_online_and_broadcast(Some(true));

                return res;
            }
            MethodName::Messages => {
                match self.user_id {
                    None => return Ok(()),
                    Some(user_id) => {
                        let mut dm = self.dm.lock().unwrap();
                        let resp = SocketMessage {
                            method: MethodName::Messages,
                            data: serde_json::to_string(&dm.add_message(req.data, user_id))
                                .unwrap(),
                        };
                        self.sender
                            .broadcast(serde_json::to_string(&resp).unwrap())?
                    }
                };
            }
            MethodName::Logout => {
                return match self.user_id {
                    None => Ok(()),
                    Some(user_id) => {
                        self.change_online_and_broadcast(Some(false));
                        self.dm.lock().unwrap().delete_token(user_id);
                        self.sender.close(ws::CloseCode::Normal)
                    }
                };
            }
            _ => {}
        }
        Ok(())
    }
}
