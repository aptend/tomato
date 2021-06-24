mod tomato_model;
mod inventory_model;
pub mod navitab_model;

use crate::{
    models::inventory_model::{Inventory, Task},
    process::ProcessHandle,
};

use inventory_model::InventoryModel;

use super::events::Key;
use tomato_model::Tomato;
use navitab_model::{NavitabModel, TabType};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub enum AppMsg {
    Notify(String),
}

#[derive(Clone)]
pub struct AppHanlde {
    sender: UnboundedSender<AppMsg>,
}

impl AppHanlde {
    pub fn notify(&self, msg: String) {
        let _ = self.sender.send(AppMsg::Notify(msg));
    }
}

#[derive(Default)]
pub struct AppBuilder {}

impl AppBuilder {
    pub fn build(self) -> App {
        let (sender, receiver) = unbounded_channel();

        let app_handle = AppHanlde { sender };

        let process_handle = ProcessHandle::new(app_handle.clone());

        let inventory = InventoryModel {
            task_selected: vec![None, None],
            inventory_selected: None,
            inventory_list: vec![
                Inventory {
                    name: "Code".to_owned(),
                    color: tui::style::Color::Blue,
                },
                Inventory {
                    name: "English".to_owned(),
                    color: tui::style::Color::Magenta,
                },
            ],
            tasks_list: vec![
                vec![
                    Task {
                        name: "Rust".to_owned(),
                        tomato_minutes: 120,
                        crate_date: "2021-6-1".to_owned(),
                        notes: "".to_owned(),
                    },
                    Task {
                        name: "Golang".to_owned(),
                        tomato_minutes: 98,
                        crate_date: "2021-4-8".to_owned(),
                        notes: "".to_owned(),
                    },
                ],
                Vec::new(),
            ],
        };

        App {
            receiver,
            process_handle: process_handle.clone(),
            active_blocks: Vec::new(),
            inventory,
            ongoing_tomato_idx: None,
            tomato: Tomato::new(app_handle, process_handle),
            tabs: NavitabModel::new(),
            notify: None,
        }
    }
}

type InventoryLocaction = (usize, usize);
pub struct App {
    pub receiver: UnboundedReceiver<AppMsg>,
    pub process_handle: ProcessHandle,
    active_blocks: Vec<ActiveBlock>,
    pub inventory: InventoryModel,
    pub ongoing_tomato_idx: Option<InventoryLocaction>,
    pub tomato: Tomato,
    pub tabs: NavitabModel,
    pub notify: Option<String>,
}

impl App {
    pub fn process_msg(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::Notify(s) => self.notify(s),
        }
    }

    fn notify(&mut self, msg: String) {
        self.notify = Some(msg);
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

    pub fn on_key(&mut self, key: Key) {
        if key == Key::Esc && self.notify.is_some() {
            self.notify.take();
            return;
        }
        match self.active_block() {
            ActiveBlock::Navitab => navi_handle(self, key),
            ActiveBlock::InventoryList => inventory_list_handle(self, key),
            ActiveBlock::InventoryTaskList => inventory_task_handle(self, key),
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
                app.push_block(ActiveBlock::InventoryTaskList);
            }
        }
        Key::Esc => app.pop_block(),
        _ => {}
    }
}

fn inventory_task_handle(app: &mut App, key: Key) {
    match key {
        Key::Up => app.inventory.previous_task(),
        Key::Down => app.inventory.next_task(),
        Key::Right | Key::Char('\n') => {
            if let Some(loc) = app.inventory.get_task_location() {
                app.ongoing_tomato_idx = Some(loc);
                app.reset_block();
                app.tabs.next();
            }
        }
        Key::Esc | Key::Left => app.pop_block(),
        _ => {}
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActiveBlock {
    Navitab,
    InventoryList,
    InventoryTaskList,
}
