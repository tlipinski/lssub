use crate::ui::actions::Action;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
use crate::ui::pad::BlockTitlePadExt;

#[derive(Debug)]
pub struct UserWidget {
    pub requests: i32,
    pub remaining: i32,
}

impl UserWidget {
    pub fn from() -> Self {
        UserWidget {
            requests: 0,
            remaining: 0,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let downloads = {
            let block = Block::bordered()
                .title_pad("Downloads remaining")
                .title_alignment(Alignment::Center)
                .border_set(border::PLAIN);

            let line = Line::from(format!(
                "{} of {}",
                self.remaining,
                self.requests + self.remaining
            ))
            .centered();

            Paragraph::new(line).block(block)
        };

        frame.render_widget(downloads, area);
    }
}
