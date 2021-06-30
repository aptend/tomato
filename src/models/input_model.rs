use tui::style::Color;

use crate::{
    db::{DbColor, EditInventory, EditTask, NewInventory, NewTask},
    process::{ProcessHandle, ProcessMsg},
};

use super::{AppHandle, AppMsg, Key};

pub enum InputContext {
    NewInventory(Box<NewInventory>),
    NewTask(Box<NewTask>),
    EditTask(Box<EditTask>),
    EditInventory(Box<EditInventory>),
}

pub struct InputModel {
    proc_hdl: ProcessHandle,
    app_hdl: AppHandle,
    context: Option<InputContext>,
    input: String,
}

fn parse_inv_input(input: &str) -> (&str, i32) {
    let parts: Vec<_> = input.rsplitn(2, '@').collect();
    if parts.len() == 1 {
        (parts[0], 0)
    } else {
        let color: DbColor = match parts[0] {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" => Color::Gray,
            "darkgray" => Color::DarkGray,
            "lightred" => Color::LightRed,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightblue" => Color::LightBlue,
            "lightmagenta" => Color::LightMagenta,
            "lightcyan" => Color::LightCyan,
            "white" => Color::White,
            _ => Color::Reset,
        }
        .into();
        (parts[1], color.into())
    }
}

impl InputModel {
    pub fn new(app: AppHandle, proc: ProcessHandle) -> Self {
        InputModel {
            app_hdl: app,
            proc_hdl: proc,
            context: None,
            input: String::new(),
        }
    }

    pub fn content(&self) -> &str {
        self.input.as_str()
    }

    pub fn unwrap_cxt(&self) -> &InputContext {
        self.context.as_ref().unwrap()
    }

    pub fn is_active(&self) -> bool {
        self.context.is_some()
    }

    pub fn set_context(&mut self, cxt: InputContext) {
        self.context = Some(cxt);
    }

    pub fn on_key(&mut self, key: Key) {
        match key {
            Key::Char('\n') => {
                let input = std::mem::take(&mut self.input);
                if input.is_empty() {
                    return;
                }
                let msg = match self.context.take().unwrap() {
                    InputContext::NewInventory(mut base) => {
                        let (name, color) = parse_inv_input(&input);
                        base.name = name.to_owned();
                        base.color = color.into();
                        ProcessMsg::CreateInventory(base)
                    }
                    InputContext::NewTask(mut base) => {
                        base.name = input;
                        ProcessMsg::CreateTask(base)
                    }
                    InputContext::EditInventory(mut inv) => {
                        let (name, color) = parse_inv_input(&input);
                        if !name.is_empty() {
                            inv.name = Some(name.to_owned());
                        }
                        if color != 0 {
                            inv.color = Some(color);
                        }
                        ProcessMsg::UpdateInventory(inv)
                    }
                    InputContext::EditTask(mut task) => {
                        task.name = Some(input);
                        ProcessMsg::UpdateTask(task)
                    }
                };
                self.app_hdl.send(AppMsg::InputEnd);
                self.proc_hdl.send(msg);
            }
            Key::Char(c) => self.input.push(c),
            Key::Backspace => {
                self.input.pop();
            }
            Key::Esc => {
                self.context = None;
                self.app_hdl.send(AppMsg::InputEnd);
            }
            _ => {}
        }
    }
}
