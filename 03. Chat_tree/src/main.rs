extern crate rand;
extern crate uuid;

use std::env;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use rand::Rng;
use uuid::Uuid;
// mod tree_node;
// use self::tree_node::TreeNode;

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

    loop {
        let mut buf = [0u8; 16];
        match socket.recv_from(&mut buf) {
            Ok((_, src_addr)) => {
                println!("Received '{}'", Uuid::from_bytes(buf));
                let rand_number = rand_generator.gen_range(0, 100);
                if rand_number > recv_fail_chance {
                    socket
                        .send_to("ok".as_bytes(), src_addr)
                        .expect("sending 'ok' error");

                    if None == childs.iter().find(|&child| *child == src_addr) {
                        println!("{} added to broadcasting group", src_addr);
                        childs.push(src_addr);
                    }

                    childs = broadcast(&buf, &socket, &mut childs, Some(src_addr));
                }
            }
            Err(_) => { /* timeout */ }
        };

        let rand_number = rand_generator.gen_range(0, 100);
        if rand_number > 90 {
            let message = Uuid::new_v4();
            println!("Broadcasing '{}' started", message);
            childs = broadcast(message.as_bytes(), &socket, &mut childs, None);
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn broadcast(
    message: &[u8],
    socket: &UdpSocket,
    childs: &mut Vec<SocketAddr>,
    filter_addr: Option<SocketAddr>,
) -> Vec<SocketAddr> {
    let mut result = Vec::new();

    for child_addr in childs.clone() {
        if filter_addr.is_some() && child_addr == filter_addr.unwrap() {
            result.push(child_addr);
            continue;
        }
        if send(&socket, message, &child_addr) {
            // println!("{} complete sent", child_addr);
            result.push(child_addr);
        } else {
            println!("{} removed from broadcasting group", child_addr);
        }
    }

    result
}

fn send(socket: &UdpSocket, buf: &[u8], addr: &SocketAddr) -> bool {
    socket.send_to(buf, addr).expect("send_to error");
    let mut confirmation_raw = [0u8; 16];
    let mut atempts_left = 3;
    loop {
        match socket.recv_from(&mut confirmation_raw) {
            Ok((_, _)) => {
                return true;
            }
            Err(_) => {
                /* timeout */
                if atempts_left == 0 {
                    return false;
                }
                atempts_left -= 1;
            }
        }

        println!("Attempts left {}", atempts_left);
        thread::sleep(Duration::from_millis(1000));
    }
}
