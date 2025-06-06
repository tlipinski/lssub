use crate::ui::events::UiEvent;
use crate::ui::events::UiEvent::FileSelected;
use anyhow::Result;
use crossterm::style::Stylize;
use log::error;
use ratatui::Frame;
use ratatui::crossterm::event::Event::Key;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui_explorer::{FileExplorer, Theme};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Explorer {
    file_explorer: FileExplorer,
    ui_tx: Sender<UiEvent>,
    pub active: bool,
}

impl Explorer {
    pub fn new(ui_tx: Sender<UiEvent>) -> Result<Explorer> {
        let theme = Theme::default()
            .with_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
            )
            .add_default_title();
        let explorer1 = FileExplorer::with_theme(theme)?;
        let explorer = Explorer {
            file_explorer: explorer1,
            ui_tx,
            active: false,
        };
        Ok(explorer)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) -> () {
        frame.render_widget(&self.file_explorer.widget(), area)
    }

    // todo maybe new message should be returned instead of sending it from here?
    pub fn handle_key_event(&mut self, event: Event) -> Result<()> {
        if let Key(key_event) = event {
            if key_event.code == KeyCode::Enter {
                self.ui_tx
                    .send(FileSelected(self.file_explorer.current().name().into()))?;
                Ok(())
            } else {
                self.file_explorer.handle(&event);
                Ok(())
            }
        } else {
            self.file_explorer.handle(&event);
            Ok(())
        }
    }
}
