use serde_json;
use std::net::SocketAddr;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub sender_name: String,
    pub received_from: Option<SocketAddr>,
    pub content: String,
}

impl Message {
    pub fn new(sender_name: String) -> Message {
        Message {
            sender_name: sender_name,
            received_from: None,
            content: Uuid::new_v4().to_string(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_json(json: String) -> Message {
        serde_json::from_str(&json).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Message;
    #[test]
    fn json_test() {
        let message = Message::new(String::from("SENDER"));
        let json = message.to_json();
        let parsed_message = Message::from_json(json);
        assert!(message.sender_name == parsed_message.sender_name);
        assert!(message.content == parsed_message.content);
        assert!(message.received_from == message.received_from);

        let s = r#"{"sender_name":"noder","received_from":null,"content":"0a9b6e0c-2d24-4f4b-a6df-e700cad34739"}"#;
        let message = Message::from_json(s.to_string());
        assert!(message.sender_name == "noder");
        assert!(message.received_from == None);
    }
}
