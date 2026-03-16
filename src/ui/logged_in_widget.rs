use crate::osb::login::Credentials;
use crate::osb::user_info::UserInfo;
use crate::secret::clear;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, Multi, UserLoggedOut};
use crate::ui::pad::BlockTitlePadExt;
use crate::ui::task_runner::{Task, TaskRunner};
use anyhow::Result;
use crossterm::event::KeyModifiers;
use log::info;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::text::Span;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

pub struct LoggedInWidget {
    pub user_info: UserInfo,
    task_runner: TaskRunner,
}

impl LoggedInWidget {
    pub fn from(user_info: UserInfo, task_runner: TaskRunner) -> Self {
        LoggedInWidget {
            user_info,
            task_runner,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title_pad("Account")
            .border_set(border::PLAIN);

        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(2),
                Constraint::Fill(1),
            ])
            .split(area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(10), Constraint::Length(3)])
            .split(block.inner(outer_layout[1]));

        let buttons_block = Block::default().title(
            Line::from(vec![
                Span::from("Logout").bold(),
                Span::from(" [Ctrl+O]  "),
                Span::from("Cancel").bold(),
                Span::from(" [Esc]"),
            ])
            .right_aligned(),
        );

        let already_logged = Paragraph::new(vec![
            Line::from(format!("Username: {}", self.user_info.data.username)),
            Line::from(format!("Level: {}", self.user_info.data.level)),
            Line::from(format!(
                "Allowed downloads: {}",
                self.user_info.data.allowed_downloads
            )),
            Line::from(format!(
                "Downloads count: {}",
                self.user_info.data.downloads_count
            )),
            Line::from(format!(
                "Remaining downloads: {}",
                self.user_info.data.remaining_downloads
            )),
        ])
        .block(Block::bordered());

        frame.render_widget(block, area);
        frame.render_widget(already_logged.centered(), layout[0]);
        frame.render_widget(buttons_block, layout[1]);
    }

    pub fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        info!("key event: {:?}", event);
        if let Event::Key(key_event) = event {
            match (key_event.code, key_event.modifiers) {
                (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                    self.task_runner.run(Task::new(async move {
                        clear().await;
                        Ok(Multi(vec![
                            UserLoggedOut,
                            ChangeStatus("Logged out".to_string())
                        ]))
                    }));
                    None
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
