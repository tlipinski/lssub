use crossterm::event::KeyModifiers;
use log::info;
use crate::ui::actions::Action;
use osb::login::Credentials;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::text::Span;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug)]
pub struct AccountWidget {
    pub username: String,
}

impl AccountWidget {
    pub fn from() -> Self {
        AccountWidget {
            username: "".into(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(" Login ".to_string().bold())
            .border_set(border::THICK);

        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Length(3)])
            .split(block.inner(outer_layout[1]));

        let buttons_block = Block::default().title(
            Line::from(vec![
                Span::from("Logout").bold(),
                Span::from(" [F4]  "),
                Span::from("Cancel").bold(),
                Span::from(" [Esc]"),
            ])
            .right_aligned(),
        );

        let already_logged =
            Paragraph::new(format!("Logged in as: {}", self.username)).block(Block::bordered());

        frame.render_widget(block, area);
        frame.render_widget(already_logged, layout[0]);
        frame.render_widget(buttons_block, layout[1]);
    }

    pub fn handle_key_event(&mut self, event: Event) -> Option<Action> {
        info!("key event: {:?}", event);
        if let Event::Key(key_event) = event {
            match key_event {
                KeyEvent {
                    code: KeyCode::F(4),
                    ..
                } => Some(Action::Logout),
                _ => None,
            }
        } else {
            None
        }
    }
}
