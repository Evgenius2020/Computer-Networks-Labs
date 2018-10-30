use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use Message;
use TreeNode;

use rand::{self, Rng};

pub fn receiving_thread(
    messages_to_broadcast: Arc<Mutex<Vec<Message>>>,
    messages_to_read: Arc<Mutex<Vec<Message>>>,
    tree_node: Arc<Mutex<TreeNode>>,
    recv_fail_chance: u8,
) {
    thread::spawn(move || {
        let mut rand_generator = rand::thread_rng();

        loop {
            let mut buf = [0u8; 8192];
            let mut tree_node = tree_node.lock().unwrap();
            match tree_node.socket.recv_from(&mut buf) {
                Ok((received, src_addr)) => {
                    let message: String = String::from_utf8_lossy(&buf[..received]).to_string();
                    let mut message = Message::from_json(message);
                    // println!(
                    //     "Received '{}' from '{}'",
                    //     message.content, message.sender_name
                    // );
                    let rand_number = rand_generator.gen_range(0, 100);
                    if rand_number > recv_fail_chance {
                        tree_node
                            .socket
                            .send_to("ok".as_bytes(), src_addr)
                            .expect("sending 'ok' error");

                        if None == tree_node.childs.iter().find(|&child| *child == src_addr) {
                            println!("{} added to broadcasting group", src_addr);
                            tree_node.childs.push(src_addr);
                        }

                        message.received_from = Some(src_addr);
                        (*messages_to_broadcast.lock().unwrap()).push(message.clone());
                        (*messages_to_read.lock().unwrap()).push(message.clone());
                    }
                }
                Err(_) => { /* timeout */ }
            };
            drop(tree_node);
            thread::sleep(Duration::from_millis(500));
        }
    });
}

pub fn sending_thread(messages: Arc<Mutex<Vec<Message>>>, tree_node: Arc<Mutex<TreeNode>>) {
    thread::spawn(move || loop {
        let mut tree_node = tree_node.lock().unwrap();
        let message = messages.lock().unwrap().pop();
        if message.is_some() {
            let message = message.unwrap();
            // println!("Broadcasing '{}' started", message.to_json());
            tree_node.broadcast(message.to_json(), message.received_from)
        }
        drop(tree_node);
        thread::sleep(Duration::from_millis(100))
    });
}
