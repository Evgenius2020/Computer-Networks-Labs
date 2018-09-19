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

    let multicast_addr = SocketAddr::new(multicast_addr.into(), 1337);
    let message = b"Hello from client!";
    let mut buf = [0u8; 64];
    
    loop {    
        socket
            .send_to(message, multicast_addr)
            .expect("Send error!");        
        match socket.recv_from(&mut buf) {
            Ok((len, remote_addr)) => {
                let data = &buf[..len];
                let response = String::from_utf8_lossy(data);
                println!("Got data: {} from {}", response, remote_addr);
            }
            Err(_) => println!("Receive fail!"),
        }
        thread::sleep(Duration::from_secs(1));
    }
}
