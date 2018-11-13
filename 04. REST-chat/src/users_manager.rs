use UsersResult;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;
use LoginResult;
use User;

#[derive(Serialize, Deserialize)]
pub struct UsersManager {
    users: Vec<User>,
    tokens: HashMap<usize, String>,
    next_id: usize,
}

impl UsersManager {
    pub fn new() -> UsersManager {
        UsersManager {
            users: Vec::new(),
            tokens: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn load(filename: &str) -> Option<UsersManager> {
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

    pub fn get_or_create(&mut self, username: &str) -> Option<(User, bool)> {
        match self.get_id_by_name(username) {
            Some(user_id) => match self.tokens.get(&user_id) {
                Some(_) => None,
                None => {
                    self.tokens.insert(user_id, UsersManager::generate_uuid());
                    Some((self.get_by_id(user_id).unwrap().clone(), false))
                }
            },
            None => {
                let user = User {
                    id: self.next_id.clone(),
                    username: username.to_string(),
                    online: true,
                };
                self.tokens.insert(user.id, UsersManager::generate_uuid());
                self.users.push(user.clone());
                self.next_id += 1;
                Some((user, true))
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

    pub fn logout(&mut self, token: &str) -> bool {
        match self.get_id_by_token(token) {
            Some(user_id) => {
                self.tokens.remove(&user_id);
                let mut user = self.get_by_id(user_id).unwrap();
                user.online = false;
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
            users: self.users.clone()
        }
    }
 
    fn generate_uuid() -> String {
        Uuid::new_v4().to_string()
    }
}
