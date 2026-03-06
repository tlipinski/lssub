use crate::secret::retrieve;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    DisabledLimitSubsToId, DownloadSubs, DownloadedSubs, EnabledLimitSubsToId, Exit, FetchSubs,
    Init, LanguagesUpdated, LoggedOut, SearchQueryUpdated, SpinnerUpdate, StartSpinner,
    StopSpinner, SubsFetched, SwitchScreen,
};
use crate::ui::app::CurrentScreen::Main;
use crate::ui::downloader::{Downloaded, Downloader};
use crate::ui::languages_screen::LanguagesScreen;
use crate::ui::search_widget::SearchWidget;
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_widget::{Sub, SubsWidget};
use crate::ui::subtitles_fetcher::SubtitlesQuery;
use crate::ui::user_widget::UserWidget;
use anyhow::Result;
use osb::subtitles::SubtitlesResponse;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::path::Path;
use tokio::sync::mpsc::Sender;

pub struct SearchScreen {
    pub search_widget: SearchWidget,
    pub subs_widget: SubsWidget,
    downloader: Downloader,
}

impl SearchScreen {
    pub fn from(base_path: &Path, file_name: Option<&str>) -> Result<SearchScreen> {
        Ok(Self {
            search_widget: SearchWidget::from(file_name.unwrap_or("").into()),
            subs_widget: SubsWidget::default(),
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
                Constraint::Min(10)
            ])
            .split(area);

        self.search_widget.render(frame, layout[0]);
        self.subs_widget.render(frame, layout[1]);
    }

    pub async fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    let selected_sub = self
                        .subs_widget
                        .state
                        .selected()
                        .and_then(|selection| self.subs_widget.subs.get(selection));

                    match selected_sub {
                        Some(s) => {
                            let downloaded = self.download(s.file_id, &s.language.clone()).await?;
                            Ok(Some(DownloadedSubs(downloaded)))
                        }
                        None => Ok(None),
                    }
                }

                KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Up
                | KeyCode::Down
                | KeyCode::F(5) => Ok(self.subs_widget.handle_key_event(key_event)),
                _ => self.search_widget.handle_key_event(event).await,
            }
        } else {
            Ok(None)
        }
    }

    pub fn update_subtitles(&mut self, subtitles_response: &SubtitlesResponse) {
        self.subs_widget.update_subtitles(subtitles_response);
    }

    async fn download(&self, file_id: i64, language: &str) -> Result<Downloaded> {
        let downloader = self.downloader.clone();

        let l = language.to_owned();

        // todo simplify
        tokio::spawn(async move {
            let token_result = retrieve().await;
            match token_result {
                Ok(token_opt) => match downloader.download(token_opt, file_id, &l).await {
                    Ok(downloaded) => Ok(downloaded),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        })
        .await?
    }
}
