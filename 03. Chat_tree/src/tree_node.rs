use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

pub struct TreeNode {
    pub socket: UdpSocket,
    pub childs: Vec<SocketAddr>,
}

impl TreeNode {
    pub fn new(socket: UdpSocket, childs: Vec<SocketAddr>) -> TreeNode {
        TreeNode {
            socket: socket,
            childs: childs,
        }
    }

    pub fn broadcast(&mut self, message: String, filter_addr: Option<SocketAddr>) {
        let mut result = Vec::new();

        for child_addr in self.childs.clone() {
            if filter_addr.is_some() && child_addr == filter_addr.unwrap() {
                result.push(child_addr);
                continue;
            }

            if self.send(&message, &child_addr) {
                // println!("{} complete sent", child_addr);
                result.push(child_addr);
            } else {
                println!("{} removed from broadcasting group", child_addr);
            }
        }

        self.childs = result;
    }

    fn send(&self, message: &String, addr: &SocketAddr) -> bool {
        self.socket
            .send_to(message.as_bytes(), addr)
            .expect("send_to error");

        let mut confirmation_raw = [0u8; 16];
        let mut atempts_left = 3;
        loop {
            match self.socket.recv_from(&mut confirmation_raw) {
                Ok((_, _)) => {
                    return true;
                }
                Err(_) => {
                    /* timeout */
                    if atempts_left == 0 {
                        return false;
                    }
                }
            }

            println!("Attempts left {}", atempts_left);
            atempts_left -= 1;

            thread::sleep(Duration::from_millis(1000));
        }
    }
}
