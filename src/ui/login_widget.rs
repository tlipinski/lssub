use crate::osb::login::{Credentials, login};
use crate::osb::user_info;
use crate::osb::user_info::get_user_info;
use crate::secret::store_credentials;
use crate::secret::store_token;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, Multi, RunTask, UserLoggedIn};
use crate::ui::pad::BlockTitlePadExt;
use crate::ui::task_runner::{Task, TaskRunner};
use anyhow::{Error, Result};
use log::{error, info, warn};
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::text::Span;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
use crate::osb::osb_client::OsbClient;
use crate::osb::values::API_URL;

pub struct LoginWidget {
    username: Input,
    password: Input,
    focus: Focus,
}

enum Focus {
    Username,
    Password,
}

impl LoginWidget {
    pub fn from() -> Self {
        LoginWidget {
            username: Input::new(String::new()),
            password: Input::new(String::new()),
            focus: Focus::Username,
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
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(block.inner(outer_layout[1]));

        let mut user_block = Block::bordered().title_pad("Username");

        let mut pass_block = Block::bordered().title_pad("Password");

        match self.focus {
            Focus::Username => {
                user_block = user_block.border_set(border::THICK);
            }
            Focus::Password => {
                pass_block = pass_block.border_set(border::THICK);
            }
        }

        let buttons = Block::default().title(
            Line::from(vec![
                Span::from("OK").bold(),
                Span::from(" [Enter]  "),
                Span::from("Cancel").bold(),
                Span::from(" [Esc]"),
            ])
            .right_aligned(),
        );

        let user_par = Paragraph::new(self.username.value()).block(user_block);

        let masked_password = "*".repeat(self.password.value().len());

        let pass_par = Paragraph::new(masked_password).block(pass_block);

        frame.render_widget(block, area);

        frame.render_widget(user_par, layout[0]);
        frame.render_widget(pass_par, layout[1]);
        frame.render_widget(buttons, layout[2]);

        match self.focus {
            Focus::Username => frame.set_cursor_position((
                layout[0].x + (self.username.visual_cursor() + 1) as u16,
                layout[0].y + 1,
            )),
            Focus::Password => frame.set_cursor_position((
                layout[1].x + (self.password.visual_cursor() + 1) as u16,
                layout[1].y + 1,
            )),
        }
    }

    pub fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        if let Event::Key(key_event) = event {
            match (key_event.code, key_event.modifiers) {
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    let credentials = Credentials {
                        username: self.username.value().to_owned(),
                        password: self.password.value().to_owned(),
                    };

                    Some(Multi(vec![
                        ChangeStatus("Logging in...".into()),
                        RunTask(Task::new("log in", LoginWidget::login_user(credentials))),
                    ]))
                }
                (KeyCode::Up, KeyModifiers::NONE) => {
                    self.focus = Focus::Username;
                    None
                }
                (KeyCode::Down, KeyModifiers::NONE) => {
                    self.focus = Focus::Password;
                    None
                }
                (KeyCode::Tab, KeyModifiers::NONE) => {
                    match self.focus {
                        Focus::Username => self.focus = Focus::Password,
                        Focus::Password => self.focus = Focus::Username,
                    }
                    None
                }
                _ => {
                    match self.focus {
                        Focus::Username => {
                            self.username.handle_event(event);
                        }
                        Focus::Password => {
                            self.password.handle_event(event);
                        }
                    }
                    None
                }
            }
        } else {
            None
        }
    }

    async fn login_user(credentials: Credentials) -> Result<Action> {
        let client = OsbClient::new(API_URL);
        match login(client, &credentials).await {
            Ok(jwt) => {
                store_credentials(credentials.clone()).await?;
                store_token(&jwt).await?;
                let user = get_user_info(&jwt).await?;
                Ok(Multi(vec![
                    UserLoggedIn(user.clone()),
                    ChangeStatus(format!("Logged in as {}", user.username)),
                ]))
            }
            Err(e) => {
                warn!("Error logging in: {}", e);
                Err(Error::msg(format!("Error logging in: {}", e)))
            }
        }
    }
}
