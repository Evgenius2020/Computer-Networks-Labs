use std::io::prelude::*;
use std::net::TcpListener;

const INSTANCE_MARKER: &str = "Instance";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        stream
            .unwrap()
            .write(String::from(INSTANCE_MARKER).as_bytes())
            .unwrap();
    }
}