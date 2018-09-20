use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;

fn join_multicast(addr: SocketAddr) -> UdpSocket {
    let local_addr : IpAddr = match addr.ip() {
        IpAddr::V4(_) => Ipv4Addr::new(0, 0, 0, 0).into(),
        IpAddr::V6(_) => Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into()
    };

    let socket =
        UdpSocket::bind(SocketAddr::new(local_addr.into(), addr.port())).expect("Bad ip addr");

    match addr.ip() {
        IpAddr::V4(ip_addr) => socket
            .join_multicast_v4(&ip_addr, &Ipv4Addr::new(0, 0, 0, 0))
            .expect("Join error"),
        IpAddr::V6(ip_addr) => socket.join_multicast_v6(&ip_addr, 0).expect("Join error"),
    };

    socket
}

fn main() {
    let multicast_addr = env::args()
        .nth(1)
        .expect("Multicast not specified!")
        .parse::<SocketAddr>()
        .expect("Failed to parse!");

    let socket = join_multicast(multicast_addr);

    let socket_copy = socket.try_clone().expect("Failed to clone socket");
    thread::spawn(move || {
        let socket = socket_copy;
        socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
        let message = b"Hello from client!";
        loop {
            socket
                .send_to(message, multicast_addr)
                .expect("Send error!");
            thread::sleep(Duration::from_secs(1));
        }
    });

    thread::spawn(move || {
        const REFRESH_DELAY: u64 = 100;
        const ACTIVITY_DURATION: i16 = 3000;
        assert!(REFRESH_DELAY > 0);
        let mut buf = [0u8; 32];
        let mut activity = HashMap::new();
        loop {
            match socket.recv_from(&mut buf) {
                Ok((_, remote_addr)) => {
                    activity.entry(remote_addr).or_insert(ACTIVITY_DURATION);
                    activity.insert(remote_addr, ACTIVITY_DURATION);
                }
                Err(_) => { /* timeout */ }
            }

            print!("{}[2J", 27 as char);
            let addrs = activity.iter_mut();
            for (addr, last_seen) in addrs {
                if *last_seen > 0 {
                    println!("{} {}", addr, last_seen)
                }
                *last_seen -= REFRESH_DELAY as i16;
            }

            thread::sleep(Duration::from_millis(REFRESH_DELAY));
        }
    });

    loop {}
}
