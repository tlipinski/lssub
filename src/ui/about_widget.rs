use crate::ui::actions::Action;
use crossterm::event::Event;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use crate::osb::values::{APP_NAME, VERSION};

pub struct AboutWidget {}

impl AboutWidget {
    pub fn new() -> AboutWidget {
        AboutWidget {}
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1)])
            .split(area);

        let about = {
            let title = "About".to_string().bold();
            let block = Block::bordered().title(title).border_set(border::THICK);
            Paragraph::new(vec![
                "".into(),
                "".into(),
                "".into(),
                Line::from(format!("{APP_NAME} v{VERSION}")).bold(),
                "".into(),
                Line::from("github: https://github.com/tlipinski/lssub"),
            ])
            .block(block)
            .centered()
        };

        frame.render_widget(about, layout[0]);
    }

    pub fn handle_key_event(&self, event: Event) -> Option<Action> {
        None
    }
}
