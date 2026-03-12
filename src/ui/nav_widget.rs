use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Line;
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct NavWidget {
    pub username: Option<String>,
}

impl NavWidget {
    pub fn new() -> NavWidget {
        Self { username: None }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
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
