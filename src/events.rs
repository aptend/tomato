use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
pub use termion::event::Key;
use termion::input::TermRead;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
    time,
};

#[derive(Clone, Copy)]
pub enum Event {
    Input(Key),
    Tick,
}

pub struct Events {
    key_recv: UnboundedReceiver<Event>,
    interval: time::Interval,
}

impl Stream for Events {
    type Item = Event;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.key_recv.poll_recv(cx) {
            Poll::Ready(key) => Poll::Ready(key),
            Poll::Pending => match self.interval.poll_tick(cx) {
                Poll::Ready(_) => Poll::Ready(Some(Event::Tick)),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}

impl Events {
    pub fn new() -> Self {
        let (tx, key_recv) = unbounded_channel();

        std::thread::spawn(move || {
            let stdin = io::stdin();
            for key in stdin.keys().flatten() {
                let _ = tx.send(Event::Input(key));
                if key == Key::Ctrl('c') {
                    break;
                }
            }
        });

        let interval = time::interval(time::Duration::from_millis(100));

        Events { key_recv, interval }
    }
}
