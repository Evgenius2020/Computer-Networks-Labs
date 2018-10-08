use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

mod tcp_receiver;

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

fn indicator(bytes_read_mutex: &Arc<Mutex<usize>>, client_addr: String, file_size: usize) {
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

pub fn handle_client_connection(stream: TcpStream) {
    thread::spawn(|| {
        let client_addr = stream.peer_addr().unwrap();
        println!("[{}]: Connected.", client_addr);
        let mut tcp_receiver = tcp_receiver::TcpReceiver::new(stream);

        let file_name_size = tcp_receiver.receive_usize().unwrap();
        let file_name = tcp_receiver.receive_str(file_name_size).unwrap();
        let file_size = tcp_receiver.receive_usize().unwrap();

        println!(
            "[{}]: Sending '{}' ({}) ...",
            client_addr,
            file_name,
            size_to_string(file_size)
        );

        const BUF_SIZE: usize = 4096;
        let bytes_read: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

        let time_start = Instant::now();
        indicator(&bytes_read, client_addr.to_string(), file_size);

        let bytes_read_mutex = Arc::clone(&bytes_read);
        {
            let mut file =
                File::create(format!("upload/{}", file_name)).expect("Failed to create file");
            loop {
                let mut bytes_read = bytes_read_mutex.lock().unwrap();
                let buf = tcp_receiver
                    .read_exact(min(BUF_SIZE, file_size - *bytes_read))
                    .unwrap();
                *bytes_read += buf.len();
                file.write_all(&buf).unwrap();
                if *bytes_read == file_size {
                    break;
                }
            }
        }

        // Small files can come in less than a second.
        let seconds_elapsed = match time_start.elapsed().as_secs() {
            0 => 1,
            seconds_elapsed => seconds_elapsed
        };
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
