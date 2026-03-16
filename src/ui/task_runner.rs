use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, StartProgress, StopProgress};
use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc::Sender;

pub struct Task {
    future: Pin<Box<dyn Future<Output = anyhow::Result<Action>> + Send + 'static>>,
}

impl Task {
    pub fn new(future: impl Future<Output = anyhow::Result<Action>> + Send + 'static) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    async fn run(self) -> anyhow::Result<Action> {
        self.future.await
    }
}

#[derive(Clone)]
pub struct TaskRunner {
    ui_tx: Sender<Action>,
}

impl TaskRunner {
    pub fn new(ui_tx: Sender<Action>) -> Self {
        Self { ui_tx }
    }

    pub fn run(&self, task: Task) {
        let ui_tx = self.ui_tx.clone();
        tokio::spawn(async move {
            ui_tx.send(StartProgress).await;
            match task.run().await {
                Ok(action) => {
                    ui_tx.send(action).await;
                    ui_tx.send(StopProgress).await;
                }
                Err(err) => {
                    ui_tx.send(ChangeStatus(err.to_string())).await;
                    ui_tx.send(StopProgress).await;
                }
            }
        });
    }
}
