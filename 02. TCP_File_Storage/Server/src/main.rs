use std::env;
use std::fs::DirBuilder;
use std::net::{SocketAddr, TcpListener};

mod handle_client;

fn main() {
    let addr = env::args()
        .nth(1)
        .expect("'server_addr':'server_port' not specified!")
        .parse::<SocketAddr>()
        .expect("Failed to parse address");

    let listener: TcpListener = TcpListener::bind(&addr).expect("Binding error");
    println!("Started listening at {}", &addr);

    DirBuilder::new().recursive(true).create("upload").unwrap();

    for stream in listener.incoming() {
        handle_client::handle_client_connection(stream.unwrap());
    }
}
