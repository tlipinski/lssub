use crate::config::{Config, ConfigProvider};
use crate::osb::get_download_link::get_download_link;
use crate::osb::login::login;
use crate::osb::user_info;
use crate::osb::user_info::{UserInfo, get_user_info};
use crate::secret::{clear, retrieve, store};
use crate::ui::about_widget::AboutWidget;
use crate::ui::account_widget::AccountWidget;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    ChangeStatus, DownloadedSubs, EnabledLimitSubsToId, Exit, FetchSubs, Init, LanguagesUpdated,
    SearchQueryUpdated, SwitchScreen, UserLoggedIn, UserLoggedOut,
};
use crate::ui::app::Action::{ReceivedInput, SubsFetched};
use crate::ui::app::CurrentScreen::{About, Account, Language, Search};
use crate::ui::downloader::Downloader;
use crate::ui::input_handler::handle_input_task;
use crate::ui::languages_widget::LanguagesWidget;
use crate::ui::logged_in_widget::LoggedInWidget;
use crate::ui::login_widget::LoginWidget;
use crate::ui::nav_widget::NavWidget;
use crate::ui::query_widget::QueryWidget;
use crate::ui::search_widget::SearchWidget;
use crate::ui::spinner::{Spinner, spinner_task};
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_list_widget::SubsListWidget;
use crate::ui::subtitles_fetcher::{SubtitlesQuery, subtitles_fetch_task};
use crate::ui::task_runner::TaskRunner;
use crate::ui::user_widget::UserWidget;
use anyhow::{Error, Result, bail};
use clap::builder::TypedValueParser;
use crossterm::event::KeyEvent;
use gio::prelude::DBusInterfaceSkeletonExt;
use log::{debug, error, info};
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};
use ratatui::{DefaultTerminal, Frame};
use std::collections::VecDeque;
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, RwLock, mpsc};
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub struct App {
    current_screen: CurrentScreen,
    search_widget: SearchWidget,
    languages_widget: LanguagesWidget,
    account_widget: AccountWidget,
    status_widget: StatusWidget,
    user_widget: UserWidget,
    nav_widget: NavWidget,
    about_widget: AboutWidget,
    features_tx: Sender<SubtitlesQuery>,
    modal_visible: bool,
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

        let spinner = Arc::new(RwLock::new(Spinner { c: ' ' }));
        let spinner_clone = spinner.clone();

        tokio::spawn(handle_input_task(ui_tx.clone(), shutdown_tx.subscribe()));
        tokio::spawn(subtitles_fetch_task(features_rx, ui_tx.clone()));
        tokio::spawn(spinner_task(spinner_clone));

        let provider = ConfigProvider::default();
        let task_runner = TaskRunner::new(ui_tx.clone());

        let task_runner_1 = task_runner.clone();
        let task_runner_2 = task_runner.clone();

        let search_screen = SearchWidget::from(base_path, file_name, task_runner_1)?;

        let mut app = App {
            search_widget: search_screen,
            current_screen: CurrentScreen::default(),
            languages_widget: LanguagesWidget::new(provider)?,
            account_widget: AccountWidget::new(task_runner_2),
            status_widget: StatusWidget::from(spinner.clone()),
            user_widget: UserWidget::from(),
            nav_widget: NavWidget::new(),
            about_widget: AboutWidget::new(),
            features_tx,
            modal_visible: false,
        };

        let mut messages = VecDeque::from([Init]);

        'main_loop: loop {
            while let Some(msg) = messages.pop_front() {
                match msg {
                    Exit => {
                        info!("Exiting application");
                        shutdown_tx.send(())?;
                        break 'main_loop;
                    }
                    _ => messages.extend(app.update(msg).await),
                }
            }

            terminal.draw(|frame| app.draw(frame))?;

            messages.extend(ui_rx.recv().await);
        }

        Ok(())
    }

    async fn update(&mut self, action: Action) -> Vec<Action> {
        debug!("action: {:?}", action);
        match action {
            ReceivedInput(event) => match self.handle_key_event(event).await {
                Ok(Some(m)) => {
                    vec![m]
                }
                Ok(None) => {
                    vec![]
                }
                Err(e) => {
                    self.status_widget.info = e.to_string();
                    vec![]
                }
            },

            SubsFetched(subtitles) => {
                self.search_widget.update_subtitles(&subtitles);
                self.status_widget.info = format!("{} results", subtitles.data.len());

                self.status_widget.in_progress = false;

                vec![]
            }

            LanguagesUpdated => {
                let languages = self.languages_widget.languages();
                let query: String = self.search_widget.query();
                vec![SwitchScreen(Search), FetchSubs(query, languages)]
            }

            UserLoggedIn(user_info) => {
                self.user_widget.requests = user_info.data.downloads_count;
                self.user_widget.remaining = user_info.data.remaining_downloads;
                self.nav_widget.username = Some(user_info.data.username.clone());

                vec![
                    SwitchScreen(Search),
                    ChangeStatus(format!("Logged in as {}", user_info.data.username)),
                ]
            }

            UserLoggedOut => {
                self.user_widget.requests = 0;
                self.user_widget.remaining = 0;
                self.nav_widget.username = None;

                vec![]
            }

            SearchQueryUpdated => {
                self.status_widget.in_progress = true;

                let languages = self.languages_widget.languages();
                let query = self.search_widget.query();
                vec![FetchSubs(query, languages)]
            }

            FetchSubs(query, languages) => {
                self.status_widget.in_progress = true;

                self.features_tx
                    .send(SubtitlesQuery {
                        query,
                        languages,
                        id: None,
                    })
                    .await;

                vec![]
            }

            Init => {
                let mut actions_res = self.account_widget.update(Init).await;

                match actions_res {
                    Ok(mut actions) => {
                        let query: String = self.search_widget.query();
                        if !query.is_empty() {
                            let languages = self.languages_widget.languages();
                            actions.push(FetchSubs(query, languages));
                        }

                        actions
                    }
                    Err(_) => {
                        self.status_widget.info = "Init error, check logs".to_string();
                        vec![]
                    }
                }
            }

            DownloadedSubs(downloaded) => {
                self.status_widget.info = format!("Downloaded: {:?}", downloaded.path);
                self.user_widget.requests = downloaded.requests;
                self.user_widget.remaining = downloaded.remaining;

                vec![]
            }

            SwitchScreen(screen) => {
                self.current_screen = screen;

                vec![]
            }

            Exit => vec![],

            EnabledLimitSubsToId(id) => {
                let languages = self.languages_widget.languages();
                let query = self.search_widget.query();
                self.features_tx
                    .send(SubtitlesQuery {
                        query,
                        languages,
                        id: Some(id),
                    })
                    .await
                    .unwrap(); // todo

                vec![]
            }

            ChangeStatus(status) => {
                self.status_widget.info = status;

                vec![]
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let content = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(area);

        let status = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(23)])
            .split(content[1]);

        self.status_widget.render(frame, status[0]);
        self.user_widget.render(frame, status[1]);
        self.nav_widget.render(frame, content[2]);

        match &self.current_screen {
            Search => {
                self.search_widget.render(frame, content[0]);
            }
            Language => {
                self.languages_widget.render(frame, content[0]);
            }
            Account => {
                self.account_widget.render(frame, content[0]);
            }
            About => {
                self.about_widget.render(frame, content[0]);
            }
        }
    }

    async fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        if (self.modal_visible) {
            if let Event::Key(key_event) = event {
                match key_event {
                    KeyEvent {
                        code: KeyCode::F(1),
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Esc, ..
                    } => match self.current_screen {
                        Search => {
                            self.search_widget.help = false;
                            self.modal_visible = false;
                            Ok(None)
                        }
                        _ => Ok(None),
                    },
                    _ => Ok(None),
                }
            } else {
                Ok(None)
            }
        } else if let Event::Key(key_event) = event {
            match key_event {
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => Ok(Some(SwitchScreen(Search))),
                KeyEvent {
                    code: KeyCode::F(1),
                    ..
                } => match self.current_screen {
                    Search => {
                        self.search_widget.help = !self.search_widget.help;
                        self.modal_visible = true;
                        Ok(None)
                    }
                    _ => Ok(None),
                },
                KeyEvent {
                    code: KeyCode::F(2),
                    ..
                } => Ok(Some(SwitchScreen(Search))),
                KeyEvent {
                    code: KeyCode::F(3),
                    ..
                } => Ok(Some(SwitchScreen(Account))),
                KeyEvent {
                    code: KeyCode::F(4),
                    ..
                } => Ok(Some(SwitchScreen(Language))),
                KeyEvent {
                    code: KeyCode::F(10),
                    ..
                } => Ok(Some(Exit)),
                KeyEvent {
                    code: KeyCode::F(12),
                    ..
                } => Ok(Some(SwitchScreen(About))),

                _ => match self.current_screen {
                    Search => self.search_widget.handle_key_event(event).await,
                    Language => self.languages_widget.handle_key_event(event),
                    Account => self.account_widget.handle_key_event(event).await,
                    About => Ok(self.about_widget.handle_key_event(event)),
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
    Search,
    Account,
    Language,
    About,
}
