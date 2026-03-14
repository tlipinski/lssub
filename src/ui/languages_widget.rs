use crate::config::{Config, ConfigProvider};
use crate::ui::actions::Action;
use crate::ui::actions::Action::{FetchSubs, LanguagesUpdated};
use crate::ui::app::Screen::Search;
use crate::ui::component::Component;
use crate::ui::pad::BlockTitlePadExt;
use Action::Init;
use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

pub struct LanguagesWidget {
    input: Input,
}

impl Component for LanguagesWidget {
    fn update(&mut self, action: &Action) -> Option<Action> {
        match action {
            Init => Some(LanguagesUpdated(self.languages())),
            _ => None,
        }
    }

    fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => Some(LanguagesUpdated(self.languages())),

                _ => {
                    self.input.handle_event(&event);
                    None
                }
            }
        } else {
            None
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(area);

        let view = {
            let block = Block::bordered()
                .title_pad("Languages (comma separated)")
                .border_set(border::PLAIN);
            let value = Line::from(self.input.value());
            Paragraph::new(value).block(block)
        };

        let x = self.input.visual_cursor();
        frame.set_cursor_position((area.x + (x + 1) as u16, area.y + 1));

        frame.render_widget(view, layout[0]);
    }
}

impl LanguagesWidget {
    pub fn new(languages: Vec<String>) -> LanguagesWidget {
        Self {
            input: Input::new(languages.join(",")),
        }
    }

    pub fn languages(&self) -> Vec<String> {
        let langs: String = self.input.value().into();
        let v = langs.split(",").collect::<Vec<&str>>();
        v.iter().map(|&x| String::from(x)).collect::<Vec<String>>()
    }
}
