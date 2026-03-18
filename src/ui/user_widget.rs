use crate::ui::actions::Action;
use crate::ui::actions::Action::SubtitleDownloaded;
use crate::ui::component::Component;
use crate::ui::pad::BlockTitlePadExt;
use Action::{UserLoggedIn, UserLoggedOut};
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug)]
pub struct UserWidget {
    requests: i32,
    remaining: i32,
}

impl Component for UserWidget {
    fn update(&mut self, action: &Action) -> Option<Action> {
        match action {
            UserLoggedIn(user) => {
                self.requests = user.downloads_count;
                self.remaining = user.remaining_downloads;
            }
            UserLoggedOut => {
                self.requests = 0;
                self.remaining = 0;
            }
            SubtitleDownloaded(downloaded) => {
                self.requests = downloaded.requests;
                self.remaining = downloaded.remaining;
            }
            _ => {}
        }

        None
    }

    fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let downloads = {
            let block = Block::bordered()
                .title_pad("Downloads remaining")
                .title_alignment(Alignment::Center)
                .border_set(border::PLAIN);

            let line = Line::from(format!(
                "{} of {}",
                self.remaining,
                self.requests + self.remaining
            ))
            .centered();

            Paragraph::new(line).block(block)
        };

        frame.render_widget(downloads, area);
    }
}

impl UserWidget {
    pub fn from() -> Self {
        UserWidget {
            requests: 0,
            remaining: 0,
        }
    }
}
