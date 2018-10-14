extern crate rand;
extern crate uuid;

use std::env;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use rand::Rng;
use uuid::Uuid;
mod tree_node;
use self::tree_node::TreeNode;

fn main() {
    let addr = env::args()
        .nth(1)
        .expect("'server_addr':'server_port' not specified!")
        .parse::<SocketAddr>()
        .expect("Failed to parse address");

    let recv_fail_chance = env::args()
        .nth(2)
        .expect("'recv_fail_chance' not specified!")
        .parse::<u8>()
        .expect("Failed to parse recv_fail_chance");

    let parent_addr: Option<SocketAddr> = match env::args().len() > 3 {
        true => Some(
            env::args()
                .nth(3)
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
    let mut rand_generator = rand::thread_rng();
    let mut childs: Vec<SocketAddr> = Vec::new();
    if parent_addr.is_some() {
        childs.push(parent_addr.unwrap());
    }

    let mut tree_node = TreeNode::new(socket, childs);

    loop {
        let mut buf = [0u8; 16];
        match tree_node.socket.recv_from(&mut buf) {
            Ok((_, src_addr)) => {
                println!("Received '{}'", Uuid::from_bytes(buf));
                let rand_number = rand_generator.gen_range(0, 100);
                if rand_number > recv_fail_chance {
                    tree_node.socket
                        .send_to("ok".as_bytes(), src_addr)
                        .expect("sending 'ok' error");

                    if None == tree_node.childs.iter().find(|&child| *child == src_addr) {
                        println!("{} added to broadcasting group", src_addr);
                        tree_node.childs.push(src_addr);
                    }

                    tree_node.broadcast(&buf, Some(src_addr));
                }
            }
            Err(_) => { /* timeout */ }
        };

        let rand_number = rand_generator.gen_range(0, 100);
        if rand_number > 90 {
            let message = Uuid::new_v4();
            println!("Broadcasing '{}' started", message);
            tree_node.broadcast(message.as_bytes(), None);
        }
        thread::sleep(Duration::from_millis(100));
    }
}