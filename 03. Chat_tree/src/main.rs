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

mod cli;
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

    let mut childs = Vec::new();
    if parent_addr.is_some() {
        childs.push(parent_addr.unwrap());
    }
    let messages_to_broadcast = Arc::new(Mutex::new(Vec::new()));
    let messages_to_read = Arc::new(Mutex::new(Vec::new()));
    let tree_node = Arc::new(Mutex::new(TreeNode::new(socket, childs)));

    threads::receiving_thread(
        messages_to_broadcast.clone(),
        messages_to_read.clone(),
        tree_node.clone(),
        recv_fail_chance,
    );
    threads::sending_thread(messages_to_broadcast.clone(), tree_node.clone());
    cli::start(
        messages_to_broadcast.clone(),
        messages_to_read.clone(),
        node_name.to_string(),
    );
}
