// this module mocks advanced status update functionality
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn send_status_update(mut stream: TcpStream) {
    let status_update = json!({
        "Status": "Active",
        "Position": 1,
        "Busy": false
    });
    while stream
        .write_all(&(status_update.to_string() + "\n").into_bytes())
        .is_ok()
    {
        thread::sleep(Duration::from_millis(500));
    }
}

fn handle_cmd(stream: TcpStream) {
    let reader = BufReader::new(&stream);
    for line in reader.lines() {
        match line {
            Ok(text) => println!("{text}"),
            Err(_) => break,
        }
    }
}

fn main() -> std::io::Result<()> {
    let status_update_listener = TcpListener::bind("127.0.0.1:5003")?;
    // accept connections and process them serially
    for stream in status_update_listener.incoming() {
        let command_stream = stream?;
        let status_update_stream = command_stream.try_clone();
        thread::spawn(|| {
            handle_cmd(command_stream);
        });
        send_status_update(status_update_stream?);
    }
    Ok(())
}
