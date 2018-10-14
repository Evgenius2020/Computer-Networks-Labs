use std::env;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

extern crate rand;
use rand::*;
// mod tree_node;
// use self::tree_node::TreeNode;

fn main() {
    let addr = env::args()
        .nth(1)
        .expect("'server_addr':'server_port' not specified!")
        .parse::<SocketAddr>()
        .expect("Failed to parse address");

    let parent_addr: Option<SocketAddr> = match env::args().len() > 2 {
        true => Some(
            env::args()
                .nth(2)
                .unwrap()
                .parse::<SocketAddr>()
                .expect("Failed to parse parent address"),
        ),
        false => None,
    };

    let socket = UdpSocket::bind(addr).expect("bind error");
    socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .expect("set timeout errror");
    let mut rand_generator = rand::thread_rng();
    let fail_chance = 0;

    let mut childs: Vec<SocketAddr> = Vec::new();
    if parent_addr.is_some() {
        childs.push(parent_addr.unwrap());
    }

    loop {
        let mut buf = [0u8; 4096];
        match socket.recv_from(&mut buf) {
            Ok((_, src_addr)) => {
                println!("received {}", String::from_utf8_lossy(&buf));
                let rand_number = rand_generator.gen_range(0, 100);
                if rand_number > fail_chance {
                    socket
                        .send_to("ok".as_bytes(), src_addr)
                        .expect("send Ok error");

                    match childs.iter().find(|&child| *child == src_addr) {
                        Some(_) => println!("{} already added", src_addr),
                        None => {
                            println!("{} added", src_addr);
                            childs.push(src_addr);
                        }
                    }

                    for child_adrr in childs.iter() {
                        if *child_adrr == src_addr {
                            continue;
                        }
                        send(&socket, &buf, child_adrr);
                    }
                }
            }
            Err(_) => { /* timeout */ }
        };

        let rand_number = rand_generator.gen_range(0, 100);
        if rand_number > 90 {
            println!("bcasting..");
            for child_adrr in childs.iter() {
                send(&socket, "shit".as_bytes(), child_adrr);
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn send(socket: &UdpSocket, buf: &[u8], addr: &SocketAddr) -> bool {
    socket.send_to(buf, addr).expect("send_to err");
    let mut confirmation_raw = [0u8; 4096];
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
        thread::sleep(Duration::from_millis(100));
    }
}
