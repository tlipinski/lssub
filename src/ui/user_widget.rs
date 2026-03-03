use crate::ui::actions::Action;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug)]
pub struct UserWidget {
    pub username: String,
    pub requests: i32,
    pub remaining: i32,
}

impl UserWidget {
    pub fn from() -> Self {
        UserWidget {
            username: "".into(),
            requests: 0,
            remaining: 0,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let user = {
            let title = "Logged in as"
                .to_string()
                .bold()
                .into_centered_line();

            let block = Block::bordered().title(title).border_set(border::THICK);

            let line = Line::from(format!(
                "{}",
                self.username
            )).centered();

            Paragraph::new(line).block(block)
        };

        let downloads = {
            let title = "Downloads remaining"
                .to_string()
                .bold()
                .into_centered_line();

            let block = Block::bordered().title(title).border_set(border::THICK);

            let line = Line::from(format!(
                "{} of {}",
                self.remaining,
                self.requests + self.remaining
            ))
            .centered();

            Paragraph::new(line).block(block)
        };

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)])
            .split(area);

        frame.render_widget(user, layout[0]);
        frame.render_widget(downloads, layout[1]);
    }
}
