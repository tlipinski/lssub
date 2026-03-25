use anyhow::Result;
use log::{error, info};
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::sleep;

pub async fn debouncer_task<F>(
    mut rx: Receiver<()>,
    duration: Duration,
    on_debounce: F,
) -> Result<()>
where
    F: AsyncFn() -> (),
{
    'outer: loop {
        sleep(duration).await;

        let mut last: Option<()> = None;

        // Receive as much as possible within outer loop cycle
        'debouncing: loop {
            match rx.try_recv() {
                Ok(request) => last = Some(request),

                Err(TryRecvError::Empty) => break 'debouncing,

                Err(TryRecvError::Disconnected) => {
                    error!("Disconnected");
                    break 'outer Ok(());
                }
            }
        }

        if last.is_some() {
            info!("Debounced");
            on_debounce().await;
        }
    }
}
