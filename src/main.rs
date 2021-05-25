use std::io;
use termion::{screen::AlternateScreen, raw::IntoRawMode};
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders, canvas::{Canvas, Line}};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::Color;

use std::sync::mpsc;

use termion::event::Key;
use termion::input::TermRead;

#[derive(Clone,Copy)]
enum Event {
    Input(Key),
    Tick
}

struct Events {
    rx: mpsc::Receiver<Event>,
    _tick_handle: std::thread::JoinHandle<()>,
    _input_handle: std::thread::JoinHandle<()>
}


impl Events {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let tx_clone = tx.clone();
        let _input_handle = std::thread::spawn(move || {
            let stdin = io::stdin();
            for evt in stdin.keys() {
                if let Ok(key) = evt {
                    if let Err(err) = tx_clone.send(Event::Input(key)) {
                        eprintln!("{}", err);
                        return;
                    }
                }
            }
        });

        let _tick_handle = std::thread::spawn(move || {
            if let Err(err) = tx.send(Event::Tick) {
                eprintln!("{}", err);
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        });

        Events {
            rx,
            _input_handle,
            _tick_handle
        }
    }

    fn next(&self) -> Event {
        self.rx.recv().unwrap()
    }
}


fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    // let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let events = Events::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(45),
                    Constraint::Percentage(10),
                    Constraint::Percentage(45)
                    ].as_ref()
                )
                .split(f.size());
                let block = Canvas::default()
                .block(Block::default().borders(Borders::ALL).title("World"))
                .marker(tui::symbols::Marker::Block)
                .paint(|ctx| {
                    // ctx.draw(&Line {
                    //     x1: 0.0,
                    //     y1: 0.0,
                    //     x2: 0.0,
                    //     y2: 50.0,
                    //     color: Color::Blue
                    // });

                    ctx.draw(&Line {
                        x1: 0.0,
                        y1: 50.0,
                        x2: 500.0,
                        y2: 50.0,
                        color: Color::Reset
                    });

                    ctx.print(0.0, 0.0, "🍅🍅🍅🍅", Color::LightGreen);
                })
                .x_bounds([00.0, 500.0])
                .y_bounds([00.0, 500.0]);
                f.render_widget(block, chunks[0]);
                let block = Block::default()
                .title("Block 2")
                .borders(Borders::ALL);
                f.render_widget(block, chunks[1]);
            })?;

            match events.next() {
                Event::Input(key) => {
                    match key {
                        Key::Char('q') => break,
                        _ => {},
                    }
                }
                Event::Tick => {}
            }
        }
    Ok(())
}
