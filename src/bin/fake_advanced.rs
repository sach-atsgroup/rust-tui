// this module mocks advanced status update functionality

use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn send_status_update(stream: TcpStream) {}

fn handle_cmd(stream: TcpStream) {
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        print!("{}", line.unwrap());
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
