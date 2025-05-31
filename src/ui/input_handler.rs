use crate::ui::app::QUIT_KEY;
use crate::ui::events::UiEvent;
use crate::ui::events::UiEvent::Input;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event::Key;
use ratatui::crossterm::event::KeyEventKind;
use std::sync::mpsc::Sender;

pub async fn handle_input_task(tx: Sender<UiEvent>) {
    loop {
        match event::read().unwrap() {
            // Key(key_event)
            //     if key_event.kind == KeyEventKind::Press && key_event.code == QUIT_KEY =>
            // {
            //     tx.send(Input(key_event));
                // Handler.abort() on this task doesn't kill it
                // Most likely it still hangs on event::read() so
                // handling QUIT_KEY explicitly breaks the loop
                // break;
            // }

            Key(key_event) if key_event.kind == KeyEventKind::Press => tx.send(Input(key_event)),

            _ => break,
        };
    }
}
