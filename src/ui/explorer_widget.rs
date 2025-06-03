use anyhow::Result;
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui_explorer::{FileExplorer, Theme};

#[derive(Debug)]
pub struct Explorer {
    file_explorer: FileExplorer,
    pub active: bool,
}

impl Explorer {
    pub fn new() -> Result<Explorer> {
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
            active: false,
        };
        Ok(explorer)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&self.file_explorer.widget(), area)
    }

    pub fn handle_key_event(&mut self, event: Event) {
        self.file_explorer.handle(&event);
    }
}
