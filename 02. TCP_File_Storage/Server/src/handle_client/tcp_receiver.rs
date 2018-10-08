use std::io::{Error, Read};
use std::mem;
use std::net::TcpStream;

pub struct TcpReceiver {
    stream: TcpStream,
}

const USIZE_SIZE: usize = mem::size_of::<usize>();

impl TcpReceiver {
    pub fn new(stream: TcpStream) -> TcpReceiver {
        TcpReceiver { stream: stream }
    }

    pub fn read_exact(&mut self, size: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; size];
        match self.stream.read_exact(&mut buf) {
            Ok(_) => Ok(buf),
            Err(e) => Err(e),
        }
    }

    pub fn receive_usize(&mut self) -> Result<usize, Error> {
        unsafe {
            match self.read_exact(USIZE_SIZE) {
                Ok(raw_size) => Ok(mem::transmute::<[u8; USIZE_SIZE], usize>(vec_to_array(
                    raw_size,
                ))),
                Err(e) => Err(e),
            }
        }
    }

    pub fn receive_str(&mut self, len: usize) -> Result<String, Error> {
        match self.read_exact(len) {
            Ok(raw_str) => Ok(String::from_utf8_lossy(&raw_str).to_string()),
            Err(e) => Err(e),
        }
    }
}

fn vec_to_array(bytes: Vec<u8>) -> [u8; 8] {
    let mut array = [0; 8];
    let bytes = &bytes[..array.len()];
    array.copy_from_slice(bytes);
    array
}
