use std::env;
use std::io::prelude::*;
use std::net::TcpStream;
use std::str;
use std::thread;
use std::time::Duration;

const INSTANCE_MARKER: &str = "Instance\n";

fn try_connect(addr: &str) -> TcpStream {
    match TcpStream::connect(&addr) {
        Err(err) => {
            println!("Instance at {}: connection failed: ({})", &addr, err);
            thread::sleep(Duration::from_secs(1));
            return try_connect(addr);
        }
        Ok(res) => {
            println!("Instance at {}: connection success", &addr);
            return res;
        }
    }
}

fn streq(str1: &str, str2: &str) -> bool {
    // println!("{} {}", str1.len(), str2.len());
    if str1.len() == str2.len() {
        return true;
    } else {
        return false;
    }
}

fn main() {
    let addrs = env::args().skip(1);
    for addr in addrs {
        thread::spawn(move || {
            let mut stream = try_connect(&addr);
            loop {
                let mut buf = [0; 64];
                stream
                    .write(String::from(INSTANCE_MARKER).as_bytes())
                    .unwrap();
                stream.flush().unwrap();
                stream.read(&mut buf).unwrap();
                let str_received = str::from_utf8(&buf).unwrap();
                if streq(INSTANCE_MARKER, str_received) {
                    println!("Instance at {}: alive", &addr);
                }
                thread::sleep(Duration::from_secs(1));
            }
        }).join()
            .unwrap();
    }
}
