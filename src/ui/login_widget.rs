use crate::ui::ui_messages::UiMessage;
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
pub struct LoginWidget {
    pub username: Input,
    pub password: Input,
    pub failed: String,
    editing: Editing,
}

#[derive(Debug)]
enum Editing {
    Username,
    Password,
}

impl LoginWidget {
    pub fn from() -> Self {
        LoginWidget {
            username: Input::new("user".into()),
            password: Input::new("pass".into()),
            failed: String::new(),
            editing: Editing::Username,
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
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(block.inner(outer_layout[1]));

        let mut user_block = Block::bordered().title(" Username ");

        let mut pass_block = Block::bordered().title(" Password ");

        match self.editing {
            Editing::Username => {
                user_block = user_block.border_set(border::THICK);
            }
            Editing::Password => {
                pass_block = pass_block.border_set(border::THICK);
            }
        }

        let buttons_block = Block::default().title(
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
        frame.render_widget(buttons_block, layout[2]);

        if (!self.failed.is_empty()) {
            let failure_par =
                Paragraph::new(self.failed.clone()).block(Block::bordered().title(" Failed "));
            frame.render_widget(failure_par, layout[3]);
        }

        match self.editing {
            Editing::Username => frame.set_cursor_position((
                layout[0].x + (self.username.visual_cursor() + 1) as u16,
                layout[0].y + 1,
            )),
            Editing::Password => frame.set_cursor_position((
                layout[1].x + (self.password.visual_cursor() + 1) as u16,
                layout[1].y + 1,
            )),
        };
    }

    pub fn handle_key_event(&mut self, event: Event) -> Option<UiMessage> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => Some(UiMessage::Login(Credentials {
                    username: self.username.value().to_owned(),
                    password: self.password.value().to_owned(),
                })),
                KeyCode::Up => {
                    self.editing = Editing::Username;
                    None
                }
                KeyCode::Down => {
                    self.editing = Editing::Password;
                    None
                }
                KeyCode::Tab => {
                    match self.editing {
                        Editing::Username => self.editing = Editing::Password,
                        Editing::Password => self.editing = Editing::Username,
                    }
                    None
                }
                _ => {
                    match self.editing {
                        Editing::Username => {
                            self.username.handle_event(&event);
                        }
                        Editing::Password => {
                            self.password.handle_event(&event);
                        }
                    }
                    None
                }
            }
        } else {
            None
        }
    }
}
