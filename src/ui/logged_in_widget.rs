use crate::secret::clear;
use crate::ui::actions::Action;
use anyhow::Result;
use crossterm::event::KeyModifiers;
use log::info;
use osb::login::Credentials;
use osb::user_info::UserInfo;
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
pub struct LoggedInWidget {
    pub user_info: UserInfo,
}

impl LoggedInWidget {
    pub fn from(user_info: UserInfo) -> Self {
        LoggedInWidget { user_info }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title("Login".to_string().bold())
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
                Span::from(" [F12]  "),
                Span::from("Cancel").bold(),
                Span::from(" [Esc]"),
            ])
            .right_aligned(),
        );

        let already_logged =
            Paragraph::new(format!("Logged in as: {}", self.user_info.data.username))
                .block(Block::bordered());

        frame.render_widget(block, area);
        frame.render_widget(already_logged, layout[0]);
        frame.render_widget(buttons_block, layout[1]);
    }

    pub async fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        info!("key event: {:?}", event);
        if let Event::Key(key_event) = event {
            match key_event {
                KeyEvent {
                    code: KeyCode::F(12),
                    ..
                } => {
                    clear().await;
                    self.user_info = UserInfo::default();
                    Ok(Some(Action::LoggedOut))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
}
