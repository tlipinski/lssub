use crate::ui::action_handler::Component;
use crate::ui::actions::Action;
use crate::ui::pad::BlockTitlePadExt;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
use Action::{UserLoggedIn, UserLoggedOut};
use crate::ui::actions::Action::DownloadedSubs;

#[derive(Debug)]
pub struct UserWidget {
    requests: i32,
    remaining: i32,
}

impl Component for UserWidget {
    fn update(&mut self, action: &Action) -> () {
        match action {
            UserLoggedIn(user_info) => {
                self.requests = user_info.data.downloads_count;
                self.remaining = user_info.data.remaining_downloads;
            }
            UserLoggedOut => {
                self.requests = 0;
                self.remaining = 0;
            }
            DownloadedSubs(downloaded) => {
                self.requests = downloaded.requests;
                self.remaining = downloaded.remaining;
            }
            _ => {}
        }
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
