use crate::secret::retrieve;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    DisabledLimitSubsToId, DownloadSubs, DownloadSubsFailed, DownloadedSubs, EnabledLimitSubsToId,
    Exit, FetchSubs, Init, LanguagesUpdated, LoggedOut, QueryUpdated, SpinnerUpdate, StartSpinner,
    StopSpinner, SwitchScreen,
};
use crate::ui::app::CurrentScreen::Main;
use crate::ui::downloader::Downloader;
use crate::ui::languages_screen::LanguagesScreen;
use crate::ui::search_widget::SearchWidget;
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_widget::SubsWidget;
use crate::ui::subtitles_fetcher::SubtitlesQuery;
use crate::ui::user_widget::UserWidget;
use anyhow::Result;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::path::Path;
use tokio::sync::mpsc::Sender;
use osb::subtitles::SubtitlesResponse;

pub struct SearchScreen {
    pub search_widget: SearchWidget,
    pub subs_widget: SubsWidget,
    pub status_widget: StatusWidget,
    downloader: Downloader,
}

impl SearchScreen {
    pub fn from(
        base_path: &Path,
        file_name: Option<&str>,
        features_tx: Sender<SubtitlesQuery>,
    ) -> Result<SearchScreen> {
        Ok(Self {
            search_widget: SearchWidget::from(file_name.unwrap_or("").into(), features_tx),
            subs_widget: SubsWidget::default(),
            status_widget: StatusWidget::from("".into()),
            downloader: Downloader::new(base_path.to_owned(), file_name.map(String::from)),
        })
    }

    async fn update(&mut self, action: Action) -> Result<Vec<Action>> {
        match action {
            _ => Ok(vec![]),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3)
            ])
            .split(area);

        self.search_widget.render(frame, layout[0]);
        self.subs_widget.render(frame, layout[1]);
        self.status_widget.render(frame, layout[2]);
    }

    pub async fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Up
                | KeyCode::Down
                | KeyCode::Enter
                | KeyCode::F(5) => Ok(self.subs_widget.handle_key_event(key_event)),
                _ => self.search_widget.handle_key_event(event).await,
            }
        } else {
            Ok(None)
        }
    }

    pub fn update_subtitles(&mut self, subtitles_response: &SubtitlesResponse) {
        self.subs_widget.update_subtitles(subtitles_response);
        self.status_widget.info = format!("{} results", subtitles_response.data.len());
    }
}
