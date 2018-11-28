use datatypes::{
    LoginResult, Message, MessageSendRequest, MessageSendResult, MessagesResult, User, UsersResult,
};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DataManager {
    users: Vec<User>,
    messages: Vec<Message>,
    tokens: HashMap<usize, String>,
    last_seen: HashMap<usize, SystemTime>,
    users_next_id: usize,
    messages_next_id: usize,
}

impl DataManager {
    pub fn new() -> DataManager {
        DataManager {
            users: Vec::new(),
            messages: Vec::new(),
            tokens: HashMap::new(),
            last_seen: HashMap::new(),
            users_next_id: 0,
            messages_next_id: 0,
        }
    }

    pub fn load(filename: &str) -> Option<DataManager> {
        match File::open(filename) {
            Ok(reader) => match serde_json::from_reader(reader) {
                Ok(um) => Some(um),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    pub fn save(&self, filename: &str) {
        let mut output = File::create(filename).unwrap();
        write!(output, "{}", serde_json::to_string(self).unwrap()).unwrap()
    }

    pub fn get_or_create(&mut self, username: &str) -> Option<(User)> {
        let token = DataManager::generate_uuid();
        match self.get_id_by_name(username) {
            Some(user_id) => match self.tokens.get(&user_id) {
                Some(_) => {
                    self.update_last_seen(user_id);
                    None
                }
                None => {
                    self.tokens.insert(user_id, token.clone());
                    self.update_last_seen(user_id);
                    Some(self.get_by_id(user_id).unwrap().clone())
                }
            },
            None => {
                let user = User {
                    id: self.users_next_id.clone(),
                    username: username.to_string(),
                    online: Some(true),
                };
                self.tokens.insert(user.id, token.clone());
                self.users.push(user.clone());
                self.update_last_seen(user.id);
                self.users_next_id += 1;
                Some(user)
            }
        }
    }

    fn get_id_by_name(&self, username: &str) -> Option<usize> {
        self.users.iter().position(|r| r.username == username)
    }

    pub fn get_id_by_token(&self, token: &str) -> Option<usize> {
        for (k, v) in self.tokens.iter() {
            if v == token {
                return Some(*k);
            }
        }
        None
    }

    pub fn get_by_id(&mut self, id: usize) -> Option<&mut User> {
        self.users.iter_mut().nth(id)
    }

    pub fn add_message(&mut self, message: MessageSendRequest, token: &str) -> MessageSendResult {
        let message = Message {
            id: self.messages_next_id,
            message: message.message,
            author: self.get_id_by_token(token).unwrap(),
        };
        self.messages.push(message.clone());
        self.messages_next_id += 1;

        MessageSendResult {
            id: message.id,
            message: message.message,
        }
    }

    pub fn generate_messages_result(&self, offset: usize, count: usize) -> MessagesResult {
        let mut result = MessagesResult {
            messages: Vec::new(),
        };

        for message in self.messages.iter() {
            if message.id >= offset && message.id < offset + count {
                result.messages.push(message.clone());
            }
        }

        result
    }

    pub fn logout(&mut self, token: &str) -> bool {
        match self.get_id_by_token(token) {
            Some(user_id) => {
                self.tokens.remove(&user_id);
                let mut user = self.get_by_id(user_id).unwrap();
                user.online = Some(false);
                true
            }
            None => false,
        }
    }

    pub fn generate_login_result(&self, user: &User) -> LoginResult {
        LoginResult {
            id: user.id,
            username: user.username.clone(),
            online: user.online,
            token: self.tokens.get(&user.id).unwrap().to_string(),
        }
    }

    pub fn generate_users_result(&self) -> UsersResult {
        UsersResult {
            users: self.users.clone(),
        }
    }

    fn generate_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn update_last_seen(&mut self, id: usize) {
        self.last_seen.insert(id, SystemTime::now());
        self.get_by_id(id).unwrap().online = Some(true);
    }

    pub fn update_online(&mut self) {
        let mut to_offline = Vec::new();
        for (id, last_seen) in self.last_seen.iter() {
            if last_seen.elapsed().unwrap() > Duration::from_secs(10) {
                to_offline.push(*id);
            }
        }

        for id in to_offline.clone() {
            self.tokens.remove_entry(&id);
            self.get_by_id(id).unwrap().online = None;
        }
    }
}
