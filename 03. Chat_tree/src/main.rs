extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate serde_json;
extern crate uuid;

use std::env;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod message;
mod threads;
mod tree_node;
use self::message::Message;
use self::tree_node::TreeNode;

fn main() {
    let addr = env::args()
        .nth(1)
        .expect("'server_addr':'server_port' not specified!")
        .parse::<SocketAddr>()
        .expect("Failed to parse address");

    let node_name = env::args().nth(2).expect("'node_name' not specified!");

    let recv_fail_chance = env::args()
        .nth(3)
        .expect("'recv_fail_chance' not specified!")
        .parse::<u8>()
        .expect("Failed to parse recv_fail_chance");

    let parent_addr: Option<SocketAddr> = match env::args().len() > 4 {
        true => Some(
            env::args()
                .nth(4)
                .unwrap()
                .parse::<SocketAddr>()
                .expect("Failed to parse parent address"),
        ),
        false => None,
    };

    let socket = UdpSocket::bind(addr).expect("bind error");
    socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .expect("set_read_timeout errror");
    let socket: Arc<Mutex<UdpSocket>> = Arc::new(Mutex::new(socket));

    let childs: Arc<Mutex<Vec<SocketAddr>>> = Arc::new(Mutex::new(Vec::new()));
    if parent_addr.is_some() {
        childs.lock().unwrap().push(parent_addr.unwrap());
    }
    let messages: Arc<Mutex<Vec<Message>>> = Arc::new(Mutex::new(Vec::new()));

    threads::messages_generating_thread(messages.clone(), node_name.to_string());
    threads::receiving_thread(
        messages.clone(),
        socket.clone(),
        childs.clone(),
        recv_fail_chance,
    );
    threads::sending_thread(messages.clone(), socket.clone(), childs.clone());

    loop {}
}