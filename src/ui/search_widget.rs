use crate::osb::subtitles::SubtitlesResponse;
use crate::secret::retrieve;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    ChangeStatus, DownloadedSubs, EnabledLimitSubsToId, Exit, FetchSubs, Init, LanguagesUpdated,
    SearchQueryUpdated, SubsFetched, SwitchScreen, UserLoggedOut,
};
use crate::ui::app::CurrentScreen::Search;
use crate::ui::downloader::{Downloaded, Downloader};
use crate::ui::languages_widget::LanguagesWidget;
use crate::ui::query_widget::QueryWidget;
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_list_widget::{Sub, SubsListWidget};
use crate::ui::subtitles_fetcher::SubtitlesQuery;
use crate::ui::user_widget::UserWidget;
use anyhow::Result;
use crossterm::event::{KeyEvent, KeyModifiers};
use log::error;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Stylize};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::path::Path;
use tokio::sync::mpsc::Sender;
use tui_popup::Popup;

pub struct SearchWidget {
    query_widget: QueryWidget,
    subs_list_widget: SubsListWidget,
    downloader: Downloader,
    pub help: bool,
}

impl SearchWidget {
    pub fn from(
        base_path: &Path,
        file_name: Option<&str>,
        ui_tx: Sender<Action>,
    ) -> Result<SearchWidget> {
        Ok(Self {
            query_widget: QueryWidget::from(file_name.unwrap_or("").into()),
            subs_list_widget: SubsListWidget::default(),
            downloader: Downloader::new(base_path.to_owned(), file_name.map(String::from), ui_tx),
            help: false,
        })
    }

    pub fn query(&self) -> String {
        self.query_widget.query()
    }

    async fn update(&mut self, action: Action) -> Result<Vec<Action>> {
        match action {
            _ => Ok(vec![]),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(area);

        self.query_widget.render(frame, layout[0]);
        self.subs_list_widget.render(frame, layout[1]);

        if (self.help) {
            let body = Text::from(vec![
                "".into(),
                Line::from("Ctrl + L: Narrow results to single feature"),
                "".into(),
            ]);
            let popup = Popup::new(body)
                .title(" Help ")
                .style(Style::new().black().on_gray());
            frame.render_widget(popup, area);
        }
    }

    pub async fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        if let Event::Key(key_event) = event {
            match key_event {
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => {
                    let selected_sub = self.subs_list_widget.selected();

                    match selected_sub {
                        Some(s) => {
                            let dn = self.downloader.clone();
                            let file_id = s.file_id;
                            let language = s.language.clone();
                            tokio::spawn(async move { dn.download(file_id, &language).await });
                            let status = format!("Downloading {}", s.title);
                            Ok(Some(ChangeStatus(status.into())))
                        }
                        None => Ok(None),
                    }
                }

                KeyEvent {
                    code: KeyCode::PageUp,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::PageDown,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Up, ..
                }
                | KeyEvent {
                    code: KeyCode::Down,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => Ok(self.subs_list_widget.handle_key_event(key_event)),
                _ => self.query_widget.handle_key_event(event).await,
            }
        } else {
            Ok(None)
        }
    }

    pub fn update_subtitles(&mut self, subtitles_response: &SubtitlesResponse) {
        self.subs_list_widget.update_subtitles(subtitles_response);
    }
}
