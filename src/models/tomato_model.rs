use std::time::{Duration, Instant};

use tui::style::{Color, Style};

use crate::process::ProcessHandle;

use super::AppHanlde;

const MINUTE_UNIT: u64 = 1;

#[derive(Debug, Clone, Copy)]
pub struct TomatoConfig {
    task_duration: u8,
    short_break_duration: u8,
    long_break_duration: u8,
    long_break_interval: u8,
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

pub struct Tomato {
    handle: AppHanlde,
    process: ProcessHandle,
    config: TomatoConfig,
    states: Vec<CountdownType>,
    step: usize,
    countdown: Countdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CountdownType {
    Focus,
    ShortBreak,
    LongBreak,
}

impl Tomato {
    pub fn new(handle: AppHanlde, process: ProcessHandle) -> Self {
        let config = TomatoConfig::default();
        let states = {
            use CountdownType::*;
            let mut states =
                [Focus, ShortBreak].repeat(config.long_break_interval.saturating_sub(1).into());
            states.extend(&[Focus, LongBreak]);
            states
        };

        let mut tomato = Tomato {
            handle,
            process,
            config,
            states,
            step: 0,
            countdown: Countdown::new(Duration::new(0, 0)),
        };

        tomato.set_focus();
        tomato
    }

    fn set_focus(&mut self) {
        let left = Duration::from_secs(self.config.task_duration as u64 * MINUTE_UNIT);
        self.countdown = Countdown::new(left).color(Color::Blue);
    }

    fn set_short_break(&mut self) {
        let left = Duration::from_secs(self.config.short_break_duration as u64 * MINUTE_UNIT);
        self.countdown = Countdown::new(left).color(Color::LightGreen);
    }

    fn set_long_break(&mut self) {
        let left = Duration::from_secs(self.config.long_break_duration as u64 * MINUTE_UNIT);
        self.countdown = Countdown::new(left).color(Color::Green);
    }

    fn switch_countdown(&mut self) {
        self.step = (self.step + 1) % self.states.len();
        match self.states[self.step] {
            CountdownType::Focus => {
                self.handle.notify("Focusing".to_owned());
                self.set_focus();
            }
            CountdownType::ShortBreak => {
                self.handle.notify("Take a short break".to_owned());
                self.set_short_break();
            }
            CountdownType::LongBreak => {
                self.handle.notify("Take a long break".to_owned());
                self.set_long_break();
            }
        }
    }

    // 生成一个 tomato row，用以发送给 process 写入数据库
    pub fn tomato_complete(&self) {
        let end_time = chrono::Utc::now().timestamp();
        let start_time = end_time - self.config.task_duration as i64 * MINUTE_UNIT as i64;

    }

    pub fn on_tick(&mut self) {
        if self.countdown.is_exhausted() {
            if self.states[self.step] == CountdownType::Focus {
                self.process.close_tomato();
            }
            self.switch_countdown();
        }
        self.countdown.on_tick();
    }

    pub fn min_and_sec(&self) -> (u64, u64) {
        self.countdown.min_and_sec()
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
