use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, StartProgress, StopProgress};
use anyhow::anyhow;
use futures_util::future::BoxFuture;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct Task {
    make_future: Arc<dyn Fn() -> BoxFuture<'static, anyhow::Result<Action>> + Send + Sync>,
}

impl Task {
    pub(crate) fn new(f: impl Future<Output = anyhow::Result<Action>> + Send + 'static) -> Self {
        let future = Arc::new(Mutex::new(Some(Box::pin(f) as BoxFuture<'static, anyhow::Result<Action>>)));

        Self {
            make_future: Arc::new(move || {
                let mut guard = future.lock().expect("Task future lock poisoned");
                guard.take().unwrap_or_else(|| {
                    Box::pin(async { Err(anyhow!("task can only be run once")) })
                })
            }),
        }
    }

    async fn run(self) -> anyhow::Result<Action> {
        (self.make_future)().await
    }
}

impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task")
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
