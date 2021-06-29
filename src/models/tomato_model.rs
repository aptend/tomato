use std::time::{Duration, Instant};

use tui::style::{Color, Style};

use crate::{db::Tomato, process::ProcessHandle};

use super::AppHandle;

#[derive(Debug, Clone, Copy)]
pub struct TomatoConfig {
    // seconds
    task_duration: u64,
    short_break_duration: u64,
    long_break_duration: u64,
    // count
    long_break_interval: usize,
}

impl Default for TomatoConfig {
    fn default() -> Self {
        TomatoConfig {
            task_duration: 5,
            short_break_duration: 3,
            long_break_duration: 10,
            long_break_interval: 3,
        }
    }
}
#[derive(Default)]
struct TomatoContext {
    where_idx: Option<(usize, usize)>,
    config: TomatoConfig,
}

struct State {
    states: Vec<CountdownType>,
    idx: usize,
}

impl State {
    fn new(long_break_interval: usize) -> Self {
        use CountdownType::*;
        let mut states = [Focus, ShortBreak].repeat(long_break_interval.saturating_sub(1).into());
        states.extend(&[Focus, LongBreak]);
        State { states, idx: 0 }
    }

    pub fn next(&mut self) -> CountdownType {
        self.idx = (self.idx + 1) % self.states.len();
        self.states[self.idx]
    }

    pub fn current(&self) -> CountdownType {
        self.states[self.idx]
    }
}

pub struct TomatoModel {
    handle: AppHandle,
    _process: ProcessHandle,
    context: TomatoContext,
    state: State,
    countdown: Countdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CountdownType {
    Focus,
    ShortBreak,
    LongBreak,
}

impl TomatoModel {
    pub fn new(handle: AppHandle, process: ProcessHandle) -> Self {
        let context = TomatoContext::default();
        let state = State::new(context.config.long_break_interval);

        let mut tomato = TomatoModel {
            handle,
            _process: process,
            context,
            state,
            countdown: Countdown::new(Duration::ZERO),
        };

        tomato.set_focus();
        tomato
    }

    fn set_focus(&mut self) {
        let left = Duration::from_secs(self.context.config.task_duration);
        self.countdown = Countdown::new(left).color(Color::Blue);
    }

    fn set_short_break(&mut self) {
        let left = Duration::from_secs(self.context.config.short_break_duration);
        self.countdown = Countdown::new(left).color(Color::LightGreen);
    }

    fn set_long_break(&mut self) {
        let left = Duration::from_secs(self.context.config.long_break_duration);
        self.countdown = Countdown::new(left).color(Color::Green);
    }

    fn switch_countdown(&mut self) {
        match self.state.next() {
            CountdownType::Focus => {
                self.set_focus();
            }
            CountdownType::ShortBreak => {
                self.set_short_break();
            }
            CountdownType::LongBreak => {
                self.set_long_break();
            }
        }
    }

    // 生成一个 tomato row，用以发送给 process 写入数据库
    pub fn tomato_complete(&self) {
        let end_time = chrono::Utc::now().timestamp();
        let start_time = end_time - self.context.config.task_duration as i64;
        self.handle.close_tomato(Box::new(Tomato {
            inventory_id: 0,
            task_id: 0,
            start_time,
            end_time,
        }));
    }

    pub fn on_tick(&mut self) {
        if self.countdown.is_exhausted() {
            if self.state.current() == CountdownType::Focus {
                self.tomato_complete();
            }
            self.switch_countdown();
        }
        self.countdown.on_tick();
    }

    pub fn min_and_sec(&self) -> (u64, u64) {
        self.countdown.min_and_sec()
    }

    pub fn where_idx(&self) -> Option<(usize, usize)> {
        self.context.where_idx
    }

    pub fn set_where_idx(&mut self, loc: (usize, usize)) {
        self.context.where_idx = Some(loc);
    }

    pub fn flip(&mut self) {
        self.countdown.flip();
    }

    pub fn fg_style(&self) -> Style {
        self.countdown.color
    }

    pub fn reset(&mut self) {
        self.set_focus();
    }
}

#[derive(Debug, Clone)]
struct Countdown {
    left: Duration,
    tickpoint: Instant,
    color: Style,
    paused: bool,
}

impl Default for Countdown {
    fn default() -> Self {
        Countdown {
            left: Duration::ZERO,
            tickpoint: Instant::now(),
            color: Style::default(),
            paused: true,
        }
    }
}

impl Countdown {
    fn new(left: Duration) -> Self {
        Countdown {
            left,
            ..Default::default()
        }
    }

    fn color(mut self, color: Color) -> Self {
        self.color = Style::default().fg(color);
        self
    }

    fn on_tick(&mut self) {
        if self.is_exhausted() {
            return;
        }

        if !self.paused {
            self.left = self.left.saturating_sub(self.tickpoint.elapsed());
        }

        self.tickpoint = Instant::now();
    }

    fn is_exhausted(&self) -> bool {
        self.left == Duration::ZERO
    }

    fn flip(&mut self) {
        self.paused = !self.paused;
    }

    fn min_and_sec(&self) -> (u64, u64) {
        let secs = self.left.as_secs();
        (secs / 60, secs % 60)
    }
}
