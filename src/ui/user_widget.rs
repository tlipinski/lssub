use crate::ui::actions::Action;
use crate::ui::actions::Action::SubtitleDownloaded;
use crate::ui::app_state::AppState;
use crate::ui::component::Component;
use crate::ui::pad::BlockTitlePadExt;
use Action::{UserLoggedIn, UserLoggedOut};
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::Line;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};

#[derive(Debug)]
pub struct UserWidget {
    requests: i32,
    remaining: i32,
}

impl Component for UserWidget {
    fn update(&mut self, action: &Action, _state: AppState) -> Option<(Action, AppState)> {
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

    fn handle_key_event(&mut self, _event: &Event, _state: AppState) -> Option<(Action, AppState)> {
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
