use crate::ui::actions::Action;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, sleep};

pub async fn spinner_task(tx: Sender<Action>) -> anyhow::Result<()> {
    let spinner = ['|', '/', '-', '\\'];
    let mut pos = 0;
    loop {
        sleep(Duration::from_millis(200)).await;
        pos += 1;
        pos %= spinner.len();
        let ch = spinner[pos];
        // tx.send(Action::SpinnerUpdate(ch)).await?
    }
}
