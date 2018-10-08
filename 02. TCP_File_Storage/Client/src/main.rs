use std::cmp::min;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use std::net::TcpStream;

fn extract_filename(mut full_filename: String) -> String {
    let last_slash_index = full_filename.rfind('/').unwrap();
    full_filename.split_off(last_slash_index + 1)
}

const USIZE_SIZE: usize = mem::size_of::<usize>();

fn main() {
    let server_addr = env::args()
        .nth(1)
        .expect("'server_addr':'server_port' not specified!");
    let filename: String = env::args().nth(2).expect("Filename not specified!");
    let mut file = File::open(&filename).expect("File not found");
    let filename = extract_filename(filename);

    let mut stream = TcpStream::connect(server_addr).expect("Connection failed!");

    unsafe {
        let filename_size = mem::transmute::<usize, [u8; USIZE_SIZE]>(filename.len());
        stream.write(&filename_size).unwrap();
    }
    stream.write(filename.as_bytes()).unwrap();

    let file_size = file.metadata().unwrap().len() as usize;
    unsafe {
        let file_size = mem::transmute::<usize, [u8; USIZE_SIZE]>(file_size);
        stream.write(&file_size).unwrap();
    }

    const BUF_SIZE: usize = 256;
    let mut bytes_sent: usize = 0;
    while bytes_sent < file_size {
        let mut buf: Vec<u8> = vec![0u8; min(BUF_SIZE, file_size - bytes_sent)];
        file.read_exact(&mut buf).expect("Read error");
        bytes_sent += buf.len();
        stream.write_all(&buf).unwrap();
    }

    println!("File completely sent!");
}
