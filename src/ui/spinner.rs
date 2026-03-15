use crate::ui::actions::Action;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, sleep};

pub struct Spinner {
    pub c: char,
}

pub async fn spinner_task(spinner: Arc<RwLock<Spinner>>) -> anyhow::Result<()> {
    let chars = ['|', '/', '-', '\\'];
    let mut pos = 0;
    loop {
        sleep(Duration::from_millis(150)).await;
        pos += 1;
        pos %= chars.len();
        spinner.write().unwrap().c = chars[pos];
    }
}
