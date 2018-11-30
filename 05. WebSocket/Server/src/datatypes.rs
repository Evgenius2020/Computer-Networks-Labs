#[derive(Serialize, Deserialize, Clone)]
pub enum MethodName {
    Login,
    Logout,
    Users,
    Messages,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SocketMessage {
    pub method: MethodName,
    pub data: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: usize,
    pub username: String,
    pub online: Option<bool>,
}

#[derive(Serialize)]
pub struct LoginResult {
    pub id: usize,
    pub username: String,
    pub online: Option<bool>,
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