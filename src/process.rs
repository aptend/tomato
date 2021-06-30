use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::db::{DbUtils, EditInventory, EditTask, NewInventory, NewTask, Tomato};
use crate::models::{AppHandle, AppMsg};

pub enum ProcessMsg {
    TomatoClose(Box<Tomato>),
    CreateInventory(Box<NewInventory>),
    DeleteInventory(i32),
    UpdateInventory(Box<EditInventory>),
    CreateTask(Box<NewTask>),
    DeleteTask(i32),
    UpdateTask(Box<EditTask>),
}

#[derive(Clone)]
pub struct ProcessHandle {
    sender: UnboundedSender<ProcessMsg>,
}

#[tokio::main]
async fn run_process(mut process: Process) {
    process.run().await;
}

impl ProcessHandle {
    pub fn new(app_handle: AppHandle) -> Self {
        let (sender, receiver) = unbounded_channel();

        let process = Process {
            receiver,
            app_handle,
        };

        std::thread::spawn(move || run_process(process));

        ProcessHandle { sender }
    }

    pub fn send(&self, msg: ProcessMsg) {
        let _ = self.sender.send(msg);
    }

    pub fn close_tomato(&self, tomato: Box<Tomato>) {
        self.send(ProcessMsg::TomatoClose(tomato))
    }
}

struct Process {
    receiver: UnboundedReceiver<ProcessMsg>,
    app_handle: AppHandle,
}

impl Process {
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.process_message(msg).await;
        }
    }

    async fn process_message(&mut self, msg: ProcessMsg) {
        use ProcessMsg::*;
        match msg {
            TomatoClose(t) => handle_tomota_close(t).await,
            CreateInventory(inv) => {
                let inv = DbUtils::create_new_inventory(&inv.name, inv.color);
                self.app_handle.send(AppMsg::NewInventory(Box::new(inv)));
            }
            CreateTask(task) => {
                let task =
                    DbUtils::create_new_task(task.inventory_id, &task.name, task.notes.as_deref());
                self.app_handle.send(AppMsg::NewTask(Box::new(task)));
            }
            DeleteInventory(id) => {
                DbUtils::delete_inventory(id);
                self.app_handle.send(AppMsg::DeleteInventory(id));
            }
            DeleteTask(id) => {
                DbUtils::delete_task(id);
                self.app_handle.send(AppMsg::DeleteTask(id));
            }
            UpdateInventory(inv) => {
                DbUtils::edit_inventory(&*inv);
                self.app_handle.send(AppMsg::EditInventory(inv))
            }
            UpdateTask(task) => {
                DbUtils::edit_task(&*task);
                self.app_handle.send(AppMsg::EditTask(task))
            }
        }
    }
}

async fn handle_tomota_close(tomato: Box<Tomato>) {
    let tomato = *tomato;
    let delta_spent = tomato.end_time - tomato.start_time;
    DbUtils::update_task_spent(tomato.task_id, delta_spent);
    DbUtils::create_new_tomato(tomato);
}
