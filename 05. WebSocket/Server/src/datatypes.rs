#[derive(Serialize, Deserialize, Clone)]
pub enum MethodName {
    NameLogin,
    TokenLogin,
    Logout,
    Users,
    Messages,
    LoginResult
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