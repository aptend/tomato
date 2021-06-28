mod input_model;
mod inventory_model;
mod navitab_model;
mod tomato_model;

use crate::{
    db::{Inventory, NewInventory, NewTask, Task, Tomato},
    events::Key,
    process::{ProcessHandle, ProcessMsg},
};

use inventory_model::InventoryModel;

pub use input_model::{InputContext, InputModel};
pub use navitab_model::{NavitabModel, TabType};
pub use tomato_model::TomatoModel;

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub enum AppMsg {
    Notify(String),
    CloseTomato(Box<Tomato>),
    NewInventory(Box<Inventory>),
    NewTask(Box<Task>),
    InputEnd,
}

#[derive(Clone)]
pub struct AppHandle {
    sender: UnboundedSender<AppMsg>,
}

impl AppHandle {
    pub fn send(&self, msg: AppMsg) {
        let _ = self.sender.send(msg);
    }

    pub fn notify(&self, msg: String) {
        self.send(AppMsg::Notify(msg));
    }

    pub fn close_tomato(&self, tomato: Box<Tomato>) {
        self.send(AppMsg::CloseTomato(tomato));
    }
}

#[derive(Default)]
pub struct AppBuilder {}

impl AppBuilder {
    pub fn build(self) -> App {
        let (sender, receiver) = unbounded_channel();
        let app_handle = AppHandle { sender };
        let process_handle = ProcessHandle::new(app_handle.clone());

        App {
            receiver,
            process_handle: process_handle.clone(),
            active_blocks: Vec::new(),
            inventory: InventoryModel::new(),
            tomato: TomatoModel::new(app_handle.clone(), process_handle.clone()),
            tabs: NavitabModel::new(),
            notify: None,
            input: InputModel::new(app_handle, process_handle),
        }
    }
}

pub struct App {
    pub receiver: UnboundedReceiver<AppMsg>,
    pub process_handle: ProcessHandle,
    active_blocks: Vec<ActiveBlock>,
    pub inventory: InventoryModel,
    pub tomato: TomatoModel,
    pub tabs: NavitabModel,
    pub notify: Option<String>,
    pub input: InputModel,
}

impl App {
    pub fn process_msg(&mut self, msg: AppMsg) {
        use AppMsg::*;
        match msg {
            Notify(s) => self.notify = Some(s),
            CloseTomato(mut t) => {
                // Can't get this shit done in app.tomato, 'cause have to get info from inventory.
                // con: App is too heavy.
                if let Some((iidx, tidx)) = self.tomato.where_idx() {
                    t.inventory_id = self.inventory.inventory_list[iidx].id;

                    let task = &mut self.inventory.tasks_list[iidx][tidx];
                    t.task_id = task.id;
                    task.spent_minutes += t.end_time - t.start_time;
                }

                self.process_handle.close_tomato(t);
            }
            NewInventory(inv) => {
                self.inventory.push_new_inventory(*inv);
            }
            NewTask(task) => {
                self.inventory.push_new_task(*task);
            }
            InputEnd => self.pop_block(),
        }
    }

    pub fn on_tick(&mut self) {
        self.tomato.on_tick();
    }

    pub fn active_block(&self) -> ActiveBlock {
        *self.active_blocks.last().unwrap_or(&ActiveBlock::Navitab)
    }

    pub fn push_block(&mut self, block: ActiveBlock) {
        self.active_blocks.push(block);
    }

    pub fn pop_block(&mut self) {
        self.active_blocks.pop();
    }

    pub fn reset_block(&mut self) {
        self.active_blocks.clear();
    }

    pub fn is_q_quit_enable(&self) -> bool {
        self.active_block() != ActiveBlock::Input
    }

    pub fn on_key(&mut self, key: Key) {
        if key == Key::Esc && self.notify.is_some() {
            self.notify.take();
            return;
        }
        match self.active_block() {
            ActiveBlock::Navitab => navi_handle(self, key),
            ActiveBlock::InventoryList => inventory_list_handle(self, key),
            ActiveBlock::TaskList => inventory_task_handle(self, key),
            ActiveBlock::Input => self.input.on_key(key),
        }
    }
}

fn navi_handle(app: &mut App, key: Key) {
    match key {
        Key::Left => app.tabs.previous(),
        Key::Right => app.tabs.next(),
        _ => {}
    }

    match app.tabs.tab_type() {
        TabType::Tomato => match key {
            Key::Char(' ') => app.tomato.flip(),
            Key::Esc => app.tomato.reset(),
            _ => {}
        },

        TabType::Inventory => match key {
            Key::Char('\n') => app.push_block(ActiveBlock::InventoryList),
            _ => {}
        },

        TabType::Statistics => {}
    }
}

fn inventory_list_handle(app: &mut App, key: Key) {
    match key {
        Key::Up => app.inventory.previous_inventory(),
        Key::Down => app.inventory.next_inventory(),
        Key::Right | Key::Char('\n') => {
            if app.inventory.inventory_selected.is_some() {
                app.push_block(ActiveBlock::TaskList);
            }
        }
        Key::Esc => app.pop_block(),
        Key::Ctrl('n') => {
            app.push_block(ActiveBlock::Input);
            let inv = Box::new(NewInventory::default());
            app.input.set_context(InputContext::Inventory(inv));
        }
        Key::Char('d') => {
            if let Some(idx) = app.inventory.inventory_selected {
                let inv = app.inventory.inventory_list.remove(idx);
                app.inventory.task_selected.remove(idx);
                app.inventory.tasks_list.remove(idx);
                app.inventory.next_inventory();
                app.process_handle.send(ProcessMsg::DeleteInventory(inv.id));
            }
        }
        _ => {}
    }
}

fn inventory_task_handle(app: &mut App, key: Key) {
    match key {
        Key::Up => app.inventory.previous_task(),
        Key::Down => app.inventory.next_task(),
        Key::Char('\n') => {
            if let Some(loc) = app.inventory.get_task_location() {
                app.tomato.set_where_idx(loc);
                app.reset_block();
                app.tabs.next();
            }
        }
        Key::Esc | Key::Left => app.pop_block(),
        Key::Ctrl('n') => {
            app.push_block(ActiveBlock::Input);
            let mut task = Box::new(NewTask::default());
            let idx = app.inventory.inventory_selected.unwrap();
            task.inventory_id = app.inventory.inventory_list[idx].id;
            app.input.set_context(InputContext::Task(task));
        }
        Key::Char('d') => {
            if let Some((iidx, tidx)) = app.inventory.get_task_location() {
                let task = app.inventory.tasks_list[iidx].remove(tidx);
                app.inventory.next_task();
                app.process_handle.send(ProcessMsg::DeleteTask(task.id));
            }
        }
        _ => {}
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActiveBlock {
    Navitab,
    InventoryList,
    TaskList,
    Input,
}
