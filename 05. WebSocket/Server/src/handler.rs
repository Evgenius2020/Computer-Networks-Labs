use data_manager::DataManager;
use datatypes::{MethodName, SocketMessage};
use serde_json;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use ws;

pub struct Handler {
    sender: Arc<Mutex<ws::Sender>>,
    last_activity: Arc<Mutex<Instant>>,
    user_id: Option<usize>,
    dm: Arc<Mutex<DataManager>>,
}

impl Handler {
    pub fn new(sender: ws::Sender, dm: Arc<Mutex<DataManager>>) -> Handler {
        Handler {
            sender: Arc::new(Mutex::new(sender)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            user_id: None,
            dm: dm,
        }
    }

    fn change_online_and_broadcast(&mut self, online: Option<bool>) {
        match self.user_id {
            None => {}
            Some(user_id) => {
                println!(
                    "{}: {}",
                    user_id,
                    match online {
                        None => "null",
                        Some(online) => match online {
                            true => "true",
                            false => "false",
                        },
                    }
                );
                let mut dm = self.dm.lock().unwrap();
                let users = dm.change_online(user_id, online);
                self.sender
                    .lock()
                    .unwrap()
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
    fn on_open(&mut self, _: ws::Handshake) -> Result<(), ws::Error> {
        let sender = self.sender.clone();
        let last_activity = self.last_activity.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(10));
            if last_activity.lock().unwrap().elapsed() > Duration::from_secs(10) {
                sender.lock().unwrap().close(ws::CloseCode::Normal).unwrap();
                return;
            }
        });

        Ok(())
    }

    fn on_close(&mut self, _code: ws::CloseCode, _reason: &str) {
        self.change_online_and_broadcast(None);
    }

    fn on_message(&mut self, msg: ws::Message) -> Result<(), ws::Error> {
        *self.last_activity.lock().unwrap() = Instant::now();

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
            MethodName::NameLogin | MethodName::TokenLogin => {
                let res = {
                    let mut dm = self.dm.lock().unwrap();
                    let login_result = match req.method {
                        MethodName::NameLogin => dm.name_login(&req.data),
                        MethodName::TokenLogin => dm.token_login(&req.data),
                        _ => None,
                    };
                    let res = match login_result {
                        None => Ok(()),
                        Some(login_result) => {
                            self.user_id = Some(login_result.id);

                            let resp = SocketMessage {
                                method: MethodName::LoginResult,
                                data: serde_json::to_string(&login_result).unwrap(),
                            };
                            self.sender
                                .lock()
                                .unwrap()
                                .send(serde_json::to_string(&resp).unwrap())
                        }
                    };

                    self.sender.lock().unwrap().send(
                        serde_json::to_string(&SocketMessage {
                            method: MethodName::Messages,
                            data: serde_json::to_string(&dm.get_messages()).unwrap(),
                        }).unwrap(),
                    )?;

                    self.sender.lock().unwrap().send(
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
                            .lock()
                            .unwrap()
                            .broadcast(serde_json::to_string(&resp).unwrap())?
                    }
                };
            }
            MethodName::Logout => {
                return match self.user_id {
                    None => Ok(()),
                    Some(_) => {
                        self.change_online_and_broadcast(Some(false));
                        self.sender.lock().unwrap().close(ws::CloseCode::Normal)
                    }
                };
            }
            _ => {}
        }
        Ok(())
    }
}
