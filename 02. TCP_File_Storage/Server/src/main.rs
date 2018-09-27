use std::cmp::min;
use std::env;
use std::fs::{File, DirBuilder};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::thread;

fn read_line_without_delimeter(stream: &mut BufRead) -> String {
    let mut result = String::new();
    stream.read_line(&mut result).unwrap();
    result.pop();
    result
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
            let mut bytes_read: usize = 0;
            {
                let mut file =
                    File::create(format!("upload/{}", file_name)).expect("Failed to create file");
                while bytes_read < file_size {
                    let mut buf = vec![0u8; min(BUF_SIZE, file_size - bytes_read)];
                    stream.read_exact(&mut buf).unwrap();
                    bytes_read += buf.len();
                    println!(
                        "[{}]: {}/{} bytes received.",
                        client_addr, bytes_read, file_size
                    );
                    file.write_all(&buf).unwrap();
                }
            }

            println!("[{}]: Sent '{}'.", client_addr, file_name);
            println!("[{}]: Connection closed.", client_addr);
        });
    }
}
