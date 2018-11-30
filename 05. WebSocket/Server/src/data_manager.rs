use datatypes::{LoginResult, Message, MessagesResult, User, UsersResult};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DataManager {
    users: Vec<User>,
    messages: Vec<Message>,
    tokens: HashMap<usize, String>,
    users_next_id: usize,
    messages_next_id: usize,
    filename: String,
}

impl DataManager {
    pub fn new(filename: String) -> DataManager {
        DataManager {
            users: Vec::new(),
            messages: Vec::new(),
            tokens: HashMap::new(),
            users_next_id: 0,
            messages_next_id: 0,
            filename: filename,
        }
    }

    pub fn load(filename: &str) -> Option<DataManager> {
        match File::open(filename) {
            Ok(reader) => match serde_json::from_reader(reader) {
                Ok(dm) => Some(dm),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    pub fn save(&self) {
        let mut output = File::create(self.filename.clone()).unwrap();
        write!(output, "{}", serde_json::to_string(self).unwrap()).unwrap()
    }

    pub fn get_or_create_user(&mut self, username: &str) -> Option<(User)> {
        let token = DataManager::generate_uuid();
        let res = match self.get_id_by_name(username) {
            Some(user_id) => match self.tokens.get(&user_id) {
                Some(_) => None,
                None => {
                    self.tokens.insert(user_id, token.clone());
                    let mut user = self.get_user_by_id(user_id).unwrap();
                    user.online = Some(true);
                    Some(user.clone())
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
                self.users_next_id += 1;
                self.save();
                Some(user)
            }
        };
        if res.is_some() {
            self.save();
        }

        res
    }

    fn get_id_by_name(&self, username: &str) -> Option<usize> {
        self.users.iter().position(|r| r.username == username)
    }

    pub fn get_user_id_by_token(&self, token: &str) -> Option<usize> {
        for (k, v) in self.tokens.iter() {
            if v == token {
                return Some(*k);
            }
        }
        None
    }

    pub fn get_user_by_id(&mut self, id: usize) -> Option<&mut User> {
        self.users.iter_mut().nth(id)
    }

    pub fn add_message(&mut self, message: String, token: &str) -> MessagesResult {
        let message = Message {
            id: self.messages_next_id,
            message: message,
            author: self.get_user_id_by_token(token).unwrap(),
        };
        self.messages.push(message.clone());
        self.messages_next_id += 1;
        self.save();

        let mut to_return = Vec::new();
        to_return.push(message);
        MessagesResult {
            messages: to_return,
        }
    }

    pub fn get_messages(&self) -> MessagesResult {
        MessagesResult {
            messages: self.messages.clone(),
        }
    }

    pub fn logout(&mut self, token: &str) -> UsersResult {
        let user_id = match self.get_user_id_by_token(token) {
            Some(user_id) => {
                self.tokens.remove(&user_id);
                let mut user = self.get_user_by_id(user_id).unwrap();
                user.online = Some(false);
                Some(user_id)
            }
            None => None,
        };

        if user_id.is_some() {
            self.save();
        }
        self.get_user(user_id.unwrap())
    }

    pub fn login(&self, user: &User) -> LoginResult {
        LoginResult {
            id: user.id,
            username: user.username.clone(),
            online: user.online,
            token: self.tokens.get(&user.id).unwrap().to_string(),
        }
    }

    pub fn get_user(&mut self, id: usize) -> UsersResult {
        let mut to_return: Vec<User> = Vec::new();
        let user = self.get_user_by_id(id);
        if user.is_some() {
            to_return.push(user.unwrap().clone());
        }
        UsersResult { users: to_return }
    }

    pub fn get_users(&mut self) -> UsersResult {
        UsersResult {
            users: self.users.clone(),
        }
    }

    fn generate_uuid() -> String {
        Uuid::new_v4().to_string()
    }
}