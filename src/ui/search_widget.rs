use crate::ui::app::QUIT_KEY;
use anyhow::Result;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, TableState};
use std::sync::mpsc::Sender;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug)]
pub struct SearchWidget {
    features_tx: Sender<String>,
    pub active: bool,
    input: Input,
}

impl SearchWidget {
    pub fn from(features_tx: Sender<String>, search_text: String) -> Self {
        SearchWidget {
            features_tx,
            active: true,
            input: Input::from(search_text)
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut title = " Search ".bold();
        if self.active {
            title = title.red()
        }
        let block = Block::bordered()
            .title(title)
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let par = Line::from(self.input.value().bold());

        let view = Paragraph::new(par).block(block);
        
        let x = self.input.visual_cursor();
        frame.set_cursor_position((area.x + (x + 1) as u16, area.y + 1));

        frame.render_widget(view, area);
    }

    pub fn init(&self) -> Result<()> {
        // if !self.search_text.is_empty() {
        //     self.features_tx.send(self.search_text.clone())?;
        // }
        Ok(())
    }

    pub fn handle_key_event(&mut self, event: Event) -> Result<()> {
        self.input.handle_event(&event);
        self.features_tx.send(self.input.value().into())?;
        Ok(())
    }
}
