use advanced_tui::StatusUpdate;
use crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, List};
use ratatui::{DefaultTerminal, Frame};
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn render(frame: &mut Frame, data: &StatusUpdate) {
    let items = [
        format!("Mode: {:?}", data.mode),
        format!("State: {:?}", data.state),
        format!("Step: {}", data.step),
    ];
    let max_item_len = items.iter().map(|s| s.len()).max().unwrap_or(0) as u16;
    let [list_area] =
        Layout::vertical([Constraint::Length(items.len() as u16 + 2)]).areas(frame.area());
    let [list_area] = Layout::horizontal([Constraint::Length(max_item_len + 2)]).areas(list_area);
    frame.render_widget(List::new(items).block(Block::bordered()), list_area);
}

fn app(
    terminal: &mut DefaultTerminal,
    buffer: Arc<Mutex<Option<StatusUpdate>>>,
) -> std::io::Result<()> {
    loop {
        let guard = buffer.lock().unwrap();

        if let Some(status) = guard.as_ref() {
            terminal.draw(|frame| render(frame, status))?;
        }
        drop(guard);
        match crossterm::event::poll(Duration::from_millis(10)) {
            Err(e) => println!("{e}"),
            Ok(event_ready) => {
                if event_ready {
                    match crossterm::event::read() {
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

fn try_receive_data(receive_buf: Arc<Mutex<Option<StatusUpdate>>>) {
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
                            match serde_json::from_str::<StatusUpdate>(&received_string_buffer) {
                                Ok(status) => {
                                    let mut guard = receive_buf.lock().unwrap();
                                    *guard = Some(status);
                                }
                                Err(_) => {}
                            }
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
    let buffer: Arc<Mutex<Option<StatusUpdate>>> = Arc::new(Mutex::new(None));
    let receive_buf = Arc::clone(&buffer);
    thread::spawn(|| try_receive_data(receive_buf));
    let _ = ratatui::run(|terminal| app(terminal, buffer));
    Ok(())
}
