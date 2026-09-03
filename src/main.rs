use crossterm::event::{Event, KeyCode};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};

fn render(frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new("This is my paragraph").centered(),
        frame.area(),
    );
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
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

fn main() {
    let _ = ratatui::run(app);
}
