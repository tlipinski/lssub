use crate::ui::actions::Action;
use crate::ui::app_state::AppState;
use crate::ui::component::Component;
use crate::ui::pad::BlockTitlePadExt;
use crate::values::{APP_NAME, REPO_URL, VERSION};
use crossterm::event::Event;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};

pub struct AboutWidget {}

impl Component for AboutWidget {
    fn update(&mut self, _action: &Action, _state: AppState) -> Option<(Action, AppState)> {
        None
    }

    fn handle_key_event(&mut self, _event: &Event, _state: AppState) -> Option<(Action, AppState)> {
        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1)])
            .split(area);

        let about = {
            let block = Block::bordered()
                .title_pad("About")
                .border_set(border::PLAIN);
            Paragraph::new(vec![
                "".into(),
                "".into(),
                "".into(),
                Line::from(format!("{APP_NAME} {VERSION}")).bold(),
                "".into(),
                Line::from(REPO_URL),
            ])
            .block(block)
            .centered()
        };

        frame.render_widget(about, layout[0]);
    }
}

impl AboutWidget {
    pub fn new() -> AboutWidget {
        AboutWidget {}
    }
}
