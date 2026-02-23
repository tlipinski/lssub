use crate::ui::app::{App, QUIT_KEY};
use crate::ui::ui_messages::UiMessage;
use crate::ui::ui_messages::UiMessage::Input;
use anyhow::Result;
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
pub async fn handle_input_task(tx: Sender<UiMessage>, mut shutdown_rx: Receiver<()>) -> Result<()> {
    loop {
        if poll(Duration::from_millis(100))? {
            match event::read()? {
                key_event @ Key(_) => tx.send(Input(key_event)).await,

                _ => Ok(()),
            };
        } else if shutdown_rx.try_recv().is_ok() {
            break Ok(());
        }
    }
}
