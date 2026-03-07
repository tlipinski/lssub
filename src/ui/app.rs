use crate::ui::nav_widget::NavWidget;
use crate::config::{Config, ConfigProvider};
use crate::secret::{clear, retrieve, store};
use crate::ui::account_widget::AccountWidget;
use crate::ui::logged_in_widget::LoggedInWidget;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{DownloadedSubs, EnabledLimitSubsToId, Exit, FetchSubs, Init, LanguagesUpdated, UserLoggedIn, UserLoggedOut, SearchQueryUpdated, SpinnerUpdate, StartSpinner, StopSpinner, SwitchScreen};
use crate::ui::app::Action::{ReceivedInput, SubsFetched};
use crate::ui::app::CurrentScreen::{Account, Language, Main};
use crate::ui::downloader::Downloader;
use crate::ui::input_handler::handle_input_task;
use crate::ui::languages_widget::LanguagesWidget;
use crate::ui::login_widget::LoginWidget;
use crate::ui::search_widget::SearchWidget;
use crate::ui::query_widget::QueryWidget;
use crate::ui::spinner::spinner_task;
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_list_widget::SubsListWidget;
use crate::ui::subtitles_fetcher::{SubtitlesQuery, subtitles_fetch_task};
use crate::ui::user_widget::UserWidget;
use anyhow::{Error, Result, bail};
use clap::builder::TypedValueParser;
use gio::prelude::DBusInterfaceSkeletonExt;
use log::{debug, error, info};
use osb::get_download_link::get_download_link;
use osb::login::login;
use osb::user_info;
use osb::user_info::{UserInfo, get_user_info};
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{StatefulWidget, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::collections::VecDeque;
use std::ops::Deref;
use std::path::Path;
use std::sync::mpsc;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub const QUIT_KEY: KeyCode = KeyCode::Esc;

pub struct App {
    current_screen: CurrentScreen,
    search_widget: SearchWidget,
    languages_widget: LanguagesWidget,
    account_widget: AccountWidget,
    status_widget: StatusWidget,
    user_widget: UserWidget,
    nav_widget: NavWidget,
    ui_tx: Sender<Action>,
    features_tx: Sender<SubtitlesQuery>,
    exit: bool,
}

impl App {
    pub async fn run(
        terminal: &mut DefaultTerminal,
        base_path: &Path,
        file_name: Option<&str>,
    ) -> Result<()> {
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::channel::<Action>(100);
        let (features_tx, features_rx) = tokio::sync::mpsc::channel::<SubtitlesQuery>(100);

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(16);

        tokio::spawn(handle_input_task(ui_tx.clone(), shutdown_tx.subscribe()));
        tokio::spawn(subtitles_fetch_task(features_rx, ui_tx.clone()));
        tokio::spawn(spinner_task(ui_tx.clone()));

        let provider = ConfigProvider::default();
        let search_screen = SearchWidget::from(base_path, file_name)?;

        let mut app = App {
            search_widget: search_screen,
            current_screen: CurrentScreen::default(),
            languages_widget: LanguagesWidget::new(provider)?,
            account_widget: AccountWidget::new(),
            status_widget: StatusWidget::from("".into()),
            user_widget: UserWidget::from(),
            nav_widget: NavWidget::new(),
            ui_tx: ui_tx.clone(),
            features_tx,
            exit: false,
        };

        let mut messages = VecDeque::from([Init]);

        while !app.exit {
            while let Some(msg) = messages.pop_front() {
                match app.update(msg).await {
                    Ok(next) => messages.extend(next),
                    Err(e) => {
                        error!("Error while updating UI: {e}");
                    }
                };
            }

            terminal.draw(|frame| app.draw(frame))?;

            messages.extend(ui_rx.recv().await);
        }

        shutdown_tx.send(())?;

        Ok(())
    }

    async fn update(&mut self, action: Action) -> Result<Vec<Action>> {
        debug!("action: {:?}", action);
        match action {
            ReceivedInput(event) => {
                if let Ok(Some(m)) = self.handle_key_event(event).await {
                    Ok(vec![m])
                } else {
                    Ok(vec![])
                }
            }

            SubsFetched(subtitles) => {
                self.search_widget.update_subtitles(&subtitles);
                self.status_widget.info = format!("{} results", subtitles.data.len());
                Ok(vec![StopSpinner])
            }

            SpinnerUpdate(chr) => {
                self.status_widget.spin(chr);
                Ok(vec![])
            }

            LanguagesUpdated => {
                let languages = self.languages_widget.languages();
                let query: String = self.search_widget.query();
                Ok(vec![SwitchScreen(Main), FetchSubs(query, languages)])
            }

            UserLoggedIn => {
                match self.account_widget.user_info() {
                    Some(user_info) => {
                        self.user_widget.requests = user_info.data.downloads_count;
                        self.user_widget.remaining = user_info.data.remaining_downloads;
                        self.nav_widget.username = Some(user_info.data.username);
                    }

                    None => {}
                }

                Ok(vec![SwitchScreen(Main)])
            }

            UserLoggedOut => {
                self.user_widget.requests = 0;
                self.user_widget.remaining = 0;
                self.nav_widget.username = None;

                Ok(vec![])
            }

            SearchQueryUpdated => {
                let languages = self.languages_widget.languages();
                let query = self.search_widget.query();
                Ok(vec![FetchSubs(query, languages)])
            }

            FetchSubs(query, languages) => {
                self.features_tx
                    .send(SubtitlesQuery {
                        query,
                        languages,
                        id: None,
                    })
                    .await?;

                Ok(vec![StartSpinner])
            }

            StartSpinner => {
                self.status_widget.spinning = true;
                Ok(vec![])
            }

            StopSpinner => {
                self.status_widget.spinning = false;
                Ok(vec![])
            }

            Init => {
                let mut actions = self.account_widget.update(Init).await?;

                let query: String = self.search_widget.query();
                if !query.is_empty() {
                    let languages = self.languages_widget.languages();
                    actions.push(FetchSubs(query, languages));
                }

                Ok(actions)
            }

            DownloadedSubs(downloaded) => {
                self.status_widget.info = format!("Downloaded: {:?}", downloaded.path);
                self.user_widget.requests = downloaded.requests;
                self.user_widget.remaining = downloaded.remaining;
                Ok(vec![StopSpinner])
            }

            SwitchScreen(screen) => {
                self.current_screen = screen;
                Ok(vec![])
            }

            Exit => {
                self.exit = true;
                Ok(vec![])
            }

            EnabledLimitSubsToId(id) => {
                let languages = self.languages_widget.languages();
                let query = self.search_widget.query();
                self.features_tx
                    .send(SubtitlesQuery {
                        query,
                        languages,
                        id: Some(id),
                    })
                    .await?;
                Ok(vec![StartSpinner])
            }

        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(area);

        let status = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Length(23),
                ]
            ).split(layout[1]);

        self.status_widget.render(frame, status[0]);
        self.user_widget.render(frame, status[1]);
        self.nav_widget.render(frame, layout[2]);

        match &self.current_screen {
            Main => {
                self.search_widget.render(frame, layout[0]);
            }
            Language => {
                self.languages_widget.render(frame, layout[0]);
            }
            Account => {
                self.account_widget.render(frame, layout[0]);
            }
        }
    }

    async fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                QUIT_KEY => Ok(Some(SwitchScreen(Main))),
                KeyCode::F(2) => Ok(Some(SwitchScreen(Main))),
                KeyCode::F(3) => Ok(Some(SwitchScreen(Account))),
                KeyCode::F(4) => Ok(Some(SwitchScreen(Language))),
                KeyCode::F(10) => Ok(Some(Exit)),

                _ => match self.current_screen {
                    Main => match key_event.code {
                        _ => self.search_widget.handle_key_event(event).await,
                    },

                    Language => match key_event.code {
                        _ => self.languages_widget.handle_key_event(event),
                    },

                    Account => match key_event.code {
                        _ => self.account_widget.handle_key_event(event).await,
                    },
                },
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Default)]
pub enum CurrentScreen {
    #[default]
    Main,
    Account,
    Language,
}
