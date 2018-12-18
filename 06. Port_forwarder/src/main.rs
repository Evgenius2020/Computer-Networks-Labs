use std::env;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};

fn parse_back_addr(back_addr: &str) -> io::Result<SocketAddr> {
    let mut back_addr = back_addr.split(":");
    let back_host = back_addr.nth(0).unwrap();
    let back_port = back_addr
        .nth(0)
        .expect("Failed to parse back_addr")
        .parse::<u16>()
        .expect("Failed to parse back_port");
    let back_addr = resolve(back_host)
        .expect("Failed to resolve back_host")
        .into_iter()
        .nth(0)
        .unwrap();

    Ok(SocketAddr::new(back_addr, back_port))
}

fn resolve(host: &str) -> io::Result<Vec<IpAddr>> {
    (host, 0)
        .to_socket_addrs()
        .map(|iter| iter.map(|socket_address| socket_address.ip()).collect())
}

struct Connection {
    id: usize,
    pipe_is_broken: bool,
    front_stream: TcpStream,
    front_buf: Vec<u8>,
    back_stream: TcpStream,
    back_buf: Vec<u8>,
}

// 'front_host':'front_port' 'back_host':'back_port'
fn main() {
    let front_addr = env::args()
        .nth(1)
        .expect("'front_host':'front_port' not specified!")
        .parse::<SocketAddr>()
        .expect("Failed to parse front_addr");

    let back_addr = parse_back_addr(
        &env::args()
            .nth(2)
            .expect("'back_host':'back_port' not specified!"),
    ).expect("Failed to parse back_addr");

    let listener: TcpListener =
        TcpListener::bind(&front_addr).expect("Failed to bind at front_addr");
    println!("Started listening at {}", &front_addr);

    let mut connections: Vec<Connection> = Vec::new();
    let mut next_id: usize = 0;
    let mut buf = [0; 8192];

    listener
        .set_nonblocking(true)
        .expect("set_nonblocking error");
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(true).expect("set_nonblocking error");
                connections.push(Connection {
                    id: next_id,
                    pipe_is_broken: false,
                    front_stream: stream,
                    front_buf: Vec::new(),
                    back_stream: TcpStream::connect(back_addr)
                        .expect("Failed to connect to back_addr"),
                    back_buf: Vec::new(),
                });
                println!("[{}] Connected!", next_id);
                next_id += 1;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
            Err(e) => panic!("IO error: {}", e),
        }

        loop {
            let index = connections.iter().position(|conn| conn.pipe_is_broken);
            if index.is_none() {
                break;
            }
            let index = index.unwrap();
            println!("[{}] Connection closed!", connections[index].id);
            connections.remove(index);
        }

        for mut connection in connections.iter_mut() {
            match connection.front_stream.read(&mut buf) {
                Ok(0) => {}
                Ok(bytes_read) => {
                    println!(
                        "[{}] Received {} bytes from front!",
                        connection.id, bytes_read
                    );
                    connection.back_buf.append(&mut buf[0..bytes_read].to_vec());
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => {
                    connection.pipe_is_broken = true;
                    continue;
                }
                Err(e) => panic!("IO error: {}", e),
            };

            match connection.back_stream.write(&connection.back_buf) {
                Ok(0) => {}
                Ok(bytes_written) => {
                    println!("[{}] Sent {} bytes to back!", connection.id, bytes_written);
                    connection.back_buf =
                        connection.back_buf[bytes_written..connection.back_buf.len()].to_vec();
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => {
                    connection.pipe_is_broken = true;
                    continue;
                }
                Err(e) => panic!("IO error: {}", e),
            };

            match connection.back_stream.read(&mut buf) {
                Ok(0) => {}
                Ok(bytes_read) => {
                    println!(
                        "[{}] Received {} bytes from back!",
                        connection.id, bytes_read
                    );
                    connection
                        .front_buf
                        .append(&mut buf[0..bytes_read].to_vec());
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => {
                    connection.pipe_is_broken = true;
                    continue;
                }
                Err(e) => panic!("IO error: {}", e),
            };

            match connection.front_stream.write(&connection.front_buf) {
                Ok(0) => {}
                Ok(bytes_written) => {
                    println!("[{}] Sent {} to front!", connection.id, bytes_written);
                    connection.front_buf =
                        connection.front_buf[bytes_written..connection.front_buf.len()].to_vec();
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => {
                    connection.pipe_is_broken = true;
                    continue;
                }
                Err(e) => panic!("IO error: {}", e),
            };
        }
    }
}
