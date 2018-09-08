use std::io::prelude::*;
use std::str;
use std::env;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

const INSTANCE_MARKER: &str = "Instance";

fn main() {
    let addrs = env::args().skip(1);
    for addr in addrs {
        thread::spawn(|| {
            let mut stream = TcpStream::connect(addr).unwrap_or_else(|err| {
                panic!("Connection failed: {}", err);
            });
            loop {
                let mut buf = [0; 512];
                stream
                    .write(String::from(INSTANCE_MARKER).as_bytes())
                    .unwrap();
                stream.flush().unwrap();
                stream.read(&mut buf).unwrap();
                println!("{}", str::from_utf8(&buf).unwrap());
                thread::sleep(Duration::from_secs(1));
            }
        }).join()
            .unwrap();
    }
}
