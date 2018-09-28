use std::cmp::min;
use std::env;
use std::fs::{DirBuilder, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn read_line_without_delimeter(stream: &mut BufRead) -> String {
    let mut result = String::new();
    stream.read_line(&mut result).unwrap();
    result.pop();
    result
}

fn indicator(bytes_read: &Arc<Mutex<usize>>, client_addr: SocketAddr, file_size: usize) {
    let bytes_read = Arc::clone(&bytes_read);
    thread::spawn(move || {
        const REFRESH_INTERVAL_SECS: u64 = 3;
        let mut bytes_read_before: usize = 0;
        loop {
            thread::sleep(Duration::from_secs(REFRESH_INTERVAL_SECS));
            let bytes_read = bytes_read.lock().unwrap();

            println!(
                "[{}]: {}/{} bytes received ({} b/s).",
                client_addr,
                *bytes_read,
                file_size,
                *bytes_read - bytes_read_before / REFRESH_INTERVAL_SECS as usize
            );

            if *bytes_read == file_size {
                break;
            }
            bytes_read_before = *bytes_read;
        }
    });
}

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
        thread::spawn(|| {
            let stream = stream.unwrap();
            let client_addr = stream.peer_addr().unwrap();
            println!("[{}]: Connected.", client_addr);

            let mut stream = BufReader::new(stream);

            let file_name = read_line_without_delimeter(&mut stream);
            println!("[{}]: Sending '{}' ...", client_addr, file_name);

            let file_size = read_line_without_delimeter(&mut stream)
                .parse::<usize>()
                .expect("Failed to parse filesize");
            println!("[{}]: {} bytes left ...", client_addr, file_size);

            const BUF_SIZE: usize = 4096;
            let bytes_read: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

            indicator(&bytes_read, client_addr, file_size);

            let bytes_read = Arc::clone(&bytes_read);
            {
                let mut file =
                    File::create(format!("upload/{}", file_name)).expect("Failed to create file");
                loop {
                    let mut bytes_read = bytes_read.lock().unwrap();
                    let mut buf = vec![0u8; min(BUF_SIZE, file_size - *bytes_read)];
                    stream.read_exact(&mut buf).unwrap();
                    *bytes_read += buf.len();
                    file.write_all(&buf).unwrap();
                    if *bytes_read == file_size {
                        break;
                    }
                }
            }

            println!("[{}]: Sent '{}'.", client_addr, file_name);
            println!("[{}]: Connection closed.", client_addr);
        });
    }
}
