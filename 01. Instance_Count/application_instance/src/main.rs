use std::env;
use std::io::prelude::*;
use std::net::TcpListener;
use std::process;
use std::thread;

fn main() {
    let addr = env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: {} 'ip:port'", env::args().nth(0).unwrap());
        process::exit(1);
    });

    let listener: TcpListener = match TcpListener::bind(&addr) {
        Ok(listener) => {
            println!("Started listening at {}", &addr);
            listener
        }
        Err(err) => {
            println!("Binding error: {}", err);
            process::exit(1);
        }
    };

    for stream in listener.incoming() {
        thread::spawn(|| {
            println!("Received connection");
            let mut stream = stream.unwrap();
            let mut buf = [0; 64];
            loop {
                match stream.read(&mut buf) {
                    Err(err) => panic!("Reading error: {}", err),
                    Ok(_) => {
                        stream.write_all(&buf).unwrap_or_else(|err| {
                            panic!("Writing error: {}", err);
                        });
                    }
                }
            }
        });
    }
}
