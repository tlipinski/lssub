use crate::ui::actions::Action;
use crate::ui::actions::Action::ReceivedInput;
use crate::ui::app::App;
use anyhow::Result;
use crossterm::event::Event::Resize;
use log::info;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event::Key;
use ratatui::crossterm::event::{KeyEventKind, poll};
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

// event::read() will still block even if the application exits, so an explicit
// shutdown message has to be sent to break the loop
// Is there another way to stop event::read()?
pub async fn handle_input_task(tx: Sender<Action>, mut shutdown_rx: Receiver<()>) -> Result<()> {
    loop {
        if poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            tx.send(ReceivedInput(ev)).await;
        } else if shutdown_rx.try_recv().is_ok() {
            break Ok(());
        }
    }
}
