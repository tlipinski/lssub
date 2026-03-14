use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, StartProgress, StopProgress};
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct TaskRunner {
    ui_tx: Sender<Action>,
}

impl TaskRunner {
    pub fn new(ui_tx: Sender<Action>) -> Self {
        Self { ui_tx }
    }

    pub fn run(&self, f: impl Future<Output = anyhow::Result<Action>> + 'static + Send) {
        let ui_tx = self.ui_tx.clone();
        tokio::spawn(async move {
            ui_tx.send(StartProgress).await;
            match f.await {
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
