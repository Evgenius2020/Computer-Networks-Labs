use std::collections::HashMap;
use std::env;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

fn main() {
    let port = env::args()
        .nth(1)
        .expect("Port not specified!")
        .parse::<u16>()
        .expect("Failed to parse!");

    let local_addr = Ipv4Addr::new(0, 0, 0, 0);
    let socket = UdpSocket::bind(SocketAddr::new(local_addr.into(), port)).expect("Bad ip addr");
    let multicast_addr = Ipv4Addr::new(224, 0, 0, 123);
    socket
        .join_multicast_v4(&multicast_addr, &local_addr)
        .expect("Join error");

    let socket_copy = socket.try_clone().expect("Failed to clone socket");
    thread::spawn(move || {
        let multicast_addr = SocketAddr::new(multicast_addr.into(), 1337);
        let socket = socket_copy;
        let message = b"Hello from client!";
        loop {
            socket
                .send_to(message, multicast_addr)
                .expect("Send error!");
            thread::sleep(Duration::from_secs(1));
        }
    });

    const ACTIVITY_DURATION: i16 = 3000;
    thread::spawn(move || {
        let mut buf = [0u8; 256];
        let mut activity = HashMap::new();
        let socket = socket.try_clone().expect("Failed to clone socket");
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .expect("Failed to set read timeout");
        loop {
            match socket.recv_from(&mut buf) {
                Ok((_, remote_addr)) => {
                    activity.entry(remote_addr).or_insert(3000);
                    activity.insert(remote_addr, ACTIVITY_DURATION);
                }
                Err(_) => { /* timeout */ }
            }

            print!("{}[2J", 27 as char);
            let addrs = activity.iter_mut();
            for (addr, last_seen) in addrs {
                 *last_seen -= 50;
                if *last_seen > 0 {
                    println!("{} {}", addr, last_seen)
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    });

    loop {}
}
