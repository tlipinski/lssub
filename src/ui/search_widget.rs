use crate::ui::app::QUIT_KEY;
use anyhow::Result;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct SearchWidget {
    pub features_tx: Sender<String>,
    pub search_text: String,
    pub active: bool,
}

impl SearchWidget {
    pub fn init(&self) -> Result<()> {
        if !self.search_text.is_empty() {
            self.features_tx.send(self.search_text.clone())?;
        }
        Ok(())
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Backspace => {
                self.search_text.pop();
                self.features_tx.send(self.search_text.clone())?;
            }
            KeyCode::Char(key) => {
                self.search_text.push(key);
                self.features_tx.send(self.search_text.clone())?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Widget for &SearchWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut title = " Search ".bold();
        if self.active {
            title = title.red()
        }
        let block = Block::bordered()
            .title(title)
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let par = Line::from(self.search_text.clone().bold());

        Paragraph::new(par).block(block).render(area, buf);
    }
}
