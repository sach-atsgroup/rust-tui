use advanced_tui::{CommandType, StatusUpdate};
use core::fmt;
use crossterm::event::{Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::widgets::{Block, List, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Debug)]
enum InputMode {
    Normal,
    SelectingCommand,
}

impl fmt::Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn render(frame: &mut Frame, data: &StatusUpdate, mode: InputMode) {
    let items = [
        format!("Mode: {:?}", data.mode),
        format!("State: {:?}", data.state),
        format!("Step: {}", data.step),
    ];
    let max_item_len = items.iter().map(|s| s.len()).max().unwrap_or(0) as u16;
    let [list_area] =
        Layout::vertical([Constraint::Length(items.len() as u16 + 2)]).areas(frame.area());
    let [list_area] = Layout::horizontal([Constraint::Length(max_item_len + 2)]).areas(list_area);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(70),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
        ])
        .split(frame.area());
    frame.render_widget(List::new(items).block(Block::bordered()), list_area);
    frame.render_widget(
        Paragraph::new(mode.to_string()).alignment(Alignment::Right),
        layout[1],
    );
    let command_options = "[c] clear - [d] complete - [h] hold - [S] stop - [s] start";
    frame.render_widget(
        Paragraph::new(command_options).block(Block::bordered()),
        layout[2],
    );
}

fn handle_key_press(
    code: KeyCode,
    mode: &mut InputMode,
    command_sender: &Sender<CommandType>,
) -> bool {
    match (*mode, code) {
        (InputMode::Normal, KeyCode::Char('q')) => return true,
        (InputMode::Normal, KeyCode::Char('c')) => {
            *mode = InputMode::SelectingCommand;
        }
        (InputMode::SelectingCommand, KeyCode::Char('c')) => {
            let _ = command_sender.send(CommandType::Clear);
            *mode = InputMode::Normal;
        }
        (InputMode::SelectingCommand, KeyCode::Char('d')) => {
            let _ = command_sender.send(CommandType::Complete);
            *mode = InputMode::Normal;
        }
        (InputMode::SelectingCommand, KeyCode::Char('h')) => {
            let _ = command_sender.send(CommandType::Hold);
            *mode = InputMode::Normal;
        }
        (InputMode::SelectingCommand, KeyCode::Char('S')) => {
            let _ = command_sender.send(CommandType::Stop);
            *mode = InputMode::Normal;
        }
        (InputMode::SelectingCommand, KeyCode::Char('s')) => {
            let _ = command_sender.send(CommandType::Start);
            *mode = InputMode::Normal;
        }
        (InputMode::SelectingCommand, KeyCode::Char('u')) => {
            let _ = command_sender.send(CommandType::Unhold);
            *mode = InputMode::Normal;
        }
        (InputMode::SelectingCommand, _) => {
            *mode = InputMode::Normal;
        }
        _ => {}
    }
    return false;
}

fn app(
    terminal: &mut DefaultTerminal,
    buffer: Arc<Mutex<Option<StatusUpdate>>>,
    command_sender: Sender<CommandType>,
) -> std::io::Result<()> {
    let mut mode = InputMode::Normal;
    loop {
        let guard = buffer.lock().unwrap();

        if let Some(status) = guard.as_ref() {
            terminal.draw(|frame| render(frame, status, mode))?;
        }
        drop(guard);
        match crossterm::event::poll(Duration::from_millis(10)) {
            Err(e) => println!("{e}"),
            Ok(event_ready) => {
                if event_ready {
                    match crossterm::event::read() {
                        Err(e) => break Err(e),
                        Ok(Event::Key(key_event)) => {
                            if key_event.is_press() {
                                if handle_key_press(key_event.code, &mut mode, &command_sender) {
                                    break Ok(());
                                }
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

fn try_receive_data(
    receive_buf: Arc<Mutex<Option<StatusUpdate>>>,
    command_receiver: Receiver<CommandType>,
) {
    loop {
        let res = TcpStream::connect("127.0.0.1:5003");
        match res {
            Ok(stream) => {
                let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);
                let mut received_string_buffer = String::new();
                loop {
                    let mut command_stream = stream.try_clone().unwrap();
                    match command_receiver.try_recv() {
                        Ok(command) => {
                            let command_serialized =
                                serde_json::to_string(&command).unwrap() + "\n";
                            let _ = command_stream.write_all(command_serialized.as_bytes());
                        }
                        Err(_) => {}
                    }
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
    let (command_tx, command_rx) = mpsc::channel::<CommandType>();
    thread::spawn(|| try_receive_data(receive_buf, command_rx));
    let _ = ratatui::run(|terminal| app(terminal, buffer, command_tx));
    Ok(())
}
