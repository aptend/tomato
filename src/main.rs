#[macro_use]
extern crate diesel;

use std::io;
use termion::{raw::IntoRawMode, screen::AlternateScreen};
use tui::backend::TermionBackend;
use tui::Terminal;

mod db;
mod events;
mod models;
mod process;
mod views;
use events::{Event, Events, Key};
use models::AppBuilder;

use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    // let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut events = Events::new();
    let mut app = AppBuilder::default().build();

    loop {
        tokio::select! {
            event = events.next() => {
                match event {
                    Some(Event::Input(key)) => {
                        if app.is_q_quit_enable() && Key::Char('q') == key {
                            break;
                        }
                        app.on_key(key);
                    }
                    Some(Event::Tick) => {
                        app.on_tick();
                    }
                    _ => {}
                }
            },

            msg = app.receiver.recv() => {
                if let Some(msg) = msg {
                    app.process_msg(msg);
                    continue; // process more messages when idle
                }
            }
        }

        terminal.draw(|f| {
            views::ui::draw_app(f, &app);
        })?;
    }
    Ok(())
}
