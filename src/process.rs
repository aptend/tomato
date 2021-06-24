use std::time::Duration;

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::models::AppHanlde;

pub enum ProcessMsg {
    TomatoClose,
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
    pub fn new(app_handle: AppHanlde) -> Self {
        let (sender, receiver) = unbounded_channel();

        let process = Process {
            receiver,
            app_handle,
        };

        std::thread::spawn(move || run_process(process));

        ProcessHandle { sender }
    }

    fn send_msg(&self, msg: ProcessMsg) {
        let _ = self.sender.send(msg);
    }

    pub fn close_tomato(&self) {
        self.send_msg(ProcessMsg::TomatoClose)
    }
}


struct Process {
    receiver: UnboundedReceiver<ProcessMsg>,
    app_handle: AppHanlde,
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
            TomatoClose => handle_tomota_close(self.app_handle.clone()).await,
        }
    }
}


async fn handle_tomota_close(handle: AppHanlde) {
    tokio::time::sleep(Duration::from_secs_f32(3.2)).await;
    handle.notify("write info down to db".to_owned());
}


