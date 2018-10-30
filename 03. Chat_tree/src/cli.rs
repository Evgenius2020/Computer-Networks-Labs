use message::Message;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex};

fn read() -> String {
    let mut s = String::new();
    stdout().flush().unwrap();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");

    if Some('\n') == s.chars().next_back() {
        s.pop();
    }
    if Some('\r') == s.chars().next_back() {
        s.pop();
    }

    s
}

pub fn start(
    messages_to_broadcast: Arc<Mutex<Vec<Message>>>,
    messages_to_read: Arc<Mutex<Vec<Message>>>,
    node_name: String,
) {
    loop {
        println!("1. Send message");
        println!("2. View messages");
        println!("3. Exit");

        let request = read();
        if request == "1" {
            let message = Message::new(node_name.clone());
            (*messages_to_broadcast.lock().unwrap()).push(message);
        }
        if request == "2" {
            let mut messages = messages_to_read.lock().unwrap();
            loop {
                let message = messages.pop();
                if message.is_none() {
                    break;
                }
                let message = message.unwrap();
                println!("{} from {}", message.content, message.sender_name)
            }
        }
        if request == "3" {
            break;
        }
    }
}
