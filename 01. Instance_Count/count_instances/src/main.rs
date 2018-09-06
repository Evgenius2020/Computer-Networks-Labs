use std::io::prelude::*;
use std::net::TcpStream;

// const INSTANCE_MARKER: &str = "Instance";

fn main() {
    let mut buffer = String::new();
    let mut listener = TcpStream::connect("127.0.0.1:7878").unwrap();
    listener.read_to_string(&mut buffer).unwrap();
    println!("{}", buffer);
}