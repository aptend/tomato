use crate::{
    db::{NewInventory, NewTask},
    process::{ProcessHandle, ProcessMsg},
};

use super::{AppHandle, AppMsg, Key};

pub enum InputContext {
    Inventory(Box<NewInventory>),
    Task(Box<NewTask>),
}

pub struct InputModel {
    proc_hdl: ProcessHandle,
    app_hdl: AppHandle,
    context: Option<InputContext>,
    input: String,
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
                let name = std::mem::take(&mut self.input);
                let msg = match self.context.take().unwrap() {
                    InputContext::Inventory(mut base) => {
                        base.name = name;
                        ProcessMsg::CreateInventory(base)
                    }
                    InputContext::Task(mut base) => {
                        base.name = name;
                        ProcessMsg::CreateTask(base)
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
