use crate::ui::ui_messages::UiMessage;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

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
        let mut title = " Downloads remaining ".to_string().bold();

        let block = Block::bordered().title(title).border_set(border::THICK);

        let par = Line::from(format!(
            "{} of {}",
            self.remaining,
            self.requests + self.remaining
        )).centered();

        let view = Paragraph::new(par).block(block);

        frame.render_widget(view, area);
    }
}
