use crossterm::event::{Event, KeyCode};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn render(frame: &mut Frame, data: &str) {
    frame.render_widget(Paragraph::new(data).centered(), frame.area());
}

fn app(terminal: &mut DefaultTerminal, buffer: Arc<Mutex<String>>) -> std::io::Result<()> {
    loop {
        let guard = buffer.lock().unwrap();
        terminal.draw(|frame| render(frame, &guard))?;
        drop(guard);
        match crossterm::event::poll(Duration::from_millis(10)) {
            Err(e) => println!("{e}"),
            Ok(event_ready) => {
                if event_ready {
                    let event_result = crossterm::event::read();
                    match event_result {
                        Err(e) => break Err(e),
                        Ok(Event::Key(key_event)) => {
                            if key_event.is_press() && key_event.code == KeyCode::Char('q') {
                                break Ok(());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn try_receive_data(receive_buf: Arc<Mutex<String>>) {
    loop {
        let res = TcpStream::connect("127.0.0.1:5003");
        match res {
            Ok(stream) => {
                let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);
                let mut received_string_buffer = String::new();
                loop {
                    received_string_buffer.clear();
                    let res = reader.read_line(&mut received_string_buffer);
                    match res {
                        Ok(size) => {
                            if size == 0 {
                                break;
                            }
                            let mut guard = receive_buf.lock().unwrap();
                            guard.clear();
                            guard.push_str(&received_string_buffer);
                        }
                        Err(_) => break,
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let buffer = Arc::new(Mutex::new(String::from("nothing received")));
    let receive_buf = Arc::clone(&buffer);
    thread::spawn(|| try_receive_data(receive_buf));
    let _ = ratatui::run(|terminal| app(terminal, buffer));
    Ok(())
}
