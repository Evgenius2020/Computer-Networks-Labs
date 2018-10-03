use std::cmp::min;
use std::env;
use std::fs::{DirBuilder, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn read_line_without_delimeter(stream: &mut BufRead) -> String {
    let mut result = String::new();
    stream.read_line(&mut result).unwrap();
    result.pop();
    result
}

fn size_to_string(size: usize) -> String {
    if size < 1000 {
        format!("{} b", size)
    } else if size < 1000 * 1000 {
        format!("{:.3} Kb", size as f32 / 1000.)
    } else if size < 1000 * 1000 * 1000 {
        format!("{:.3} Mb", size as f32 / 1000000.)
    } else {
        format!("{:.3} Gb", size as f32 / 1000000000.)
    }
}

fn indicator(bytes_read_mutex: &Arc<Mutex<usize>>, client_addr: SocketAddr, file_size: usize) {
    let bytes_read_mutex = Arc::clone(&bytes_read_mutex);
    thread::spawn(move || {
        let mut bytes_read_before: usize = 0;
        loop {
            let time_start = Instant::now();
            thread::sleep(Duration::from_secs(1));
            let bytes_read = *bytes_read_mutex.lock().unwrap();
            if bytes_read == file_size {
                break;
            }

            let time_spent = time_start.elapsed().as_secs();

            let bytes_per_second: f32 = (bytes_read - bytes_read_before) as f32 / time_spent as f32;
            println!(
                "[{}]: {:.2}% received ({}/s).",
                client_addr,
                (bytes_read as f32 / file_size as f32) * 100.,
                size_to_string(bytes_per_second as usize)
            );

            bytes_read_before = bytes_read;
        }
    });
}

fn main() {
    let addr = env::args()
        .nth(1)
        .expect("'server_addr':'server_port' not specified!")
        .parse::<SocketAddr>()
        .expect("Failed to parse address");

    DirBuilder::new().recursive(true).create("upload").unwrap();

    let listener: TcpListener = TcpListener::bind(&addr).expect("Binding error");
    println!("Started listening at {}", &addr);

    for stream in listener.incoming() {
        thread::spawn(|| {
            let stream = stream.unwrap();
            let client_addr = stream.peer_addr().unwrap();
            println!("[{}]: Connected.", client_addr);

            let mut stream = BufReader::new(stream);

            let file_name = read_line_without_delimeter(&mut stream);
            let file_size = read_line_without_delimeter(&mut stream)
                .parse::<usize>()
                .expect("Failed to parse filesize");
            println!(
                "[{}]: Sending '{}' ({}) ...",
                client_addr,
                file_name,
                size_to_string(file_size)
            );

            const BUF_SIZE: usize = 4096;
            let bytes_read: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

            let time_start = Instant::now();
            indicator(&bytes_read, client_addr, file_size);

            let bytes_read_mutex = Arc::clone(&bytes_read);
            {
                let mut file =
                    File::create(format!("upload/{}", file_name)).expect("Failed to create file");
                loop {
                    let mut bytes_read = bytes_read_mutex.lock().unwrap();
                    let mut buf = vec![0u8; min(BUF_SIZE, file_size - *bytes_read)];
                    stream.read_exact(&mut buf).unwrap();
                    *bytes_read += buf.len();
                    file.write_all(&buf).unwrap();
                    if *bytes_read == file_size {
                        break;
                    }
                }
            }

            let seconds_elapsed = time_start.elapsed().as_secs();
            println!(
                "[{}]: Sent '{}' in {} seconds ({}/s).",
                client_addr,
                file_name,
                seconds_elapsed,
                size_to_string((file_size as u64 / seconds_elapsed) as usize)
            );
            println!("[{}]: Connection closed.", client_addr);
        });
    }
}
