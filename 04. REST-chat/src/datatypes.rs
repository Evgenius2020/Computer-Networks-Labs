#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: usize,
    pub username: String,
    pub online: bool,
}

#[derive(Serialize)]
pub struct LoginResult {
    pub id: usize,
    pub username: String,
    pub online: bool,
    pub token: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
}

#[derive(Serialize)]
pub struct UsersResult {
    pub users: Vec<User>,
}

#[derive(Serialize)]
pub struct MessagesResult {
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: usize,
    pub message: String,
    pub author: usize,
}

#[derive(Deserialize)]
pub struct MessageSendRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct MessageSendResult {
    pub id: usize,
    pub message: String,
}
