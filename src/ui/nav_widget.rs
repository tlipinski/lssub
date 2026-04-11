use crate::ui::actions::Action;
use crate::ui::app_state::AppState;
use crate::ui::component::Component;
use Action::{UserLoggedIn, UserLoggedOut};
use crossterm::event::Event;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Line;
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

pub struct NavWidget {
    pub username: Option<String>,
}

impl Component for NavWidget {
    fn update(&mut self, action: &Action, _state: AppState) -> Option<(Action, AppState)> {
        match action {
            UserLoggedIn(user) => {
                self.username = Some(user.username.clone());
            }
            UserLoggedOut => {
                self.username = None;
            }
            _ => {}
        }

        None
    }

    fn handle_key_event(&mut self, _event: &Event, _state: AppState) -> Option<(Action, AppState)> {
        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let main_nav = {
            let account = if let Some(u) = &self.username {
                Span::from(format!(" Account ({}) | ", u))
            } else {
                Span::from(" Account | ")
            };

            Paragraph::new(Line::from(vec![
                Span::from("F1:").bold(),
                Span::from(" Help | "),
                Span::from("F2:").bold(),
                Span::from(" Search | "),
                Span::from("F3:").bold(),
                account,
                Span::from("F4:").bold(),
                Span::from(" Languages | "),
                Span::from("F10:").bold(),
                Span::from(" Exit | "),
                Span::from("F12:").bold(),
                Span::from(" About"),
            ]))
            .centered()
        };

        frame.render_widget(main_nav, area);
    }
}

impl NavWidget {
    pub fn new() -> NavWidget {
        Self { username: None }
    }
}
