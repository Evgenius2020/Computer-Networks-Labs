use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use Message;
use TreeNode;

use rand::{self, Rng};

pub fn receiving_thread(
    messages: Arc<Mutex<Vec<Message>>>,
    socket: Arc<Mutex<UdpSocket>>,
    childs: Arc<Mutex<Vec<SocketAddr>>>,
    recv_fail_chance: u8,
) {
    thread::spawn(move || {
        let mut rand_generator = rand::thread_rng();
        let tree_node = TreeNode::new(socket.clone(), childs.clone());

        loop {
            let mut buf = [0u8; 8192];
            match tree_node.socket.lock().unwrap().recv_from(&mut buf) {
                Ok((received, src_addr)) => {
                    let message: String = String::from_utf8_lossy(&buf[..received]).to_string();
                    let mut message = Message::from_json(message);
                    println!("Received '{}' from '{}'", message.content, message.sender_name);
                    let rand_number = rand_generator.gen_range(0, 100);
                    if rand_number > recv_fail_chance {
                        tree_node
                            .socket
                            .lock()
                            .unwrap()
                            .send_to("ok".as_bytes(), src_addr)
                            .expect("sending 'ok' error");

                        if None == tree_node
                            .childs
                            .lock()
                            .unwrap()
                            .iter()
                            .find(|&child| *child == src_addr)
                        {
                            println!("{} added to broadcasting group", src_addr);
                            tree_node.childs.lock().unwrap().push(src_addr);
                        }

                        message.received_from = Some(src_addr);
                        (*messages.lock().unwrap()).push(message);
                    }
                }
                Err(_) => { /* timeout */ }
            };
        }
    });
}

pub fn messages_generating_thread(messages: Arc<Mutex<Vec<Message>>>, node_name: String) {
    thread::spawn(move || loop {
        let message = Message::new(node_name.clone());
        println!("Broadcasing '{}' started", message.to_json());
        (*messages.lock().unwrap()).push(message);
        thread::sleep(Duration::from_secs(1));
    });
}

pub fn sending_thread(
    messages: Arc<Mutex<Vec<Message>>>,
    socket: Arc<Mutex<UdpSocket>>,
    childs: Arc<Mutex<Vec<SocketAddr>>>,
) {
    thread::spawn(move || loop {
        let tree_node = TreeNode::new(socket.clone(), childs.clone());
        let message = messages.lock().unwrap().pop();
        if message.is_some() {
            let message = message.unwrap();
            tree_node.broadcast(message.to_json(), message.received_from)
        }
        thread::sleep(Duration::from_millis(100))
    });
}
