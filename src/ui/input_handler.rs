use crate::ui::app::{App, QUIT_KEY};
use crate::ui::events::UiEvent;
use crate::ui::events::UiEvent::Input;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event::Key;
use ratatui::crossterm::event::{KeyEventKind, poll};
use std::sync::mpsc::{Sender};
use std::time::Duration;
use tokio::sync::broadcast::Receiver;

// event::read() will still block even if the application exits, so an explicit 
// shutdown message has to be sent to break the loop
// Is there another way to stop event::read()?
pub async fn handle_input_task(tx: Sender<UiEvent>, mut shutdown_rx: Receiver<()>) {
    loop {
        if poll(Duration::from_millis(100)).unwrap() {
            match event::read().unwrap() {
                Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    tx.send(Input(key_event))
                }

                _ => break,
            };
        } else {
            match shutdown_rx.try_recv() {
                Ok(_) => break,
                _ => {}
            }
        }
    }
}
