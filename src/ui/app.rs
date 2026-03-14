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
    ChangeStatus, DownloadedSubs, EnabledLimitSubsToId, Exit, FeatureInfo, FetchSubs, Init,
    LanguagesUpdated, SearchQueryUpdated, SwitchScreen, Tuple, UserLoggedIn, UserLoggedOut,
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
use std::cmp::PartialEq;
use std::collections::{HashMap, VecDeque};
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, RwLock, mpsc};
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use crate::ui::action_handler::Component;

pub struct App {
    current_screen: CurrentScreen,
    search_widget: SearchWidget,
    languages_widget: LanguagesWidget,
    status_widget: StatusWidget,
    user_widget: UserWidget,
    about_widget: AboutWidget,
    subtitles_tx: Sender<SubtitlesQuery>,
    config_provider: ConfigProvider,
    modal_visible: bool,
    pub widgets: HashMap<String, Box<dyn Component>>,
}

impl App {
    pub async fn run(
        terminal: &mut DefaultTerminal,
        base_path: &Path,
        file_name: Option<&str>,
    ) -> Result<()> {
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::channel::<Action>(100);
        let (subtitles_tx, subtitles_rx) = tokio::sync::mpsc::channel::<SubtitlesQuery>(100);
        let (featurex_tx, featurex_rx) = tokio::sync::mpsc::channel::<i32>(100);

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(16);
        let task_runner = TaskRunner::new(ui_tx.clone());

        let spinner = Arc::new(RwLock::new(Spinner { c: ' ' }));
        let spinner_clone = spinner.clone();

        tokio::spawn(handle_input_task(ui_tx.clone(), shutdown_tx.subscribe()));
        tokio::spawn(subtitles_fetch_task(subtitles_rx, task_runner.clone()));
        tokio::spawn(spinner_task(spinner_clone));

        let config_provider = ConfigProvider::default();

        let search_screen = SearchWidget::from(base_path, file_name, task_runner.clone())?;

        let account_widget = AccountWidget::new(task_runner.clone());
        let nav_widget = NavWidget::new();

        let mut x: HashMap<String, Box<dyn Component>> = HashMap::new();
        x.insert("account".into(), Box::new(account_widget));
        x.insert("nav".into(), Box::new(nav_widget));

        let mut app = App {
            search_widget: search_screen,
            current_screen: CurrentScreen::default(),
            languages_widget: LanguagesWidget::new(config_provider.get_config()?.languages)?,
            status_widget: StatusWidget::from(spinner.clone()),
            user_widget: UserWidget::from(),
            about_widget: AboutWidget::new(),
            config_provider,
            subtitles_tx,
            modal_visible: false,
            widgets: x
        };

        let mut message_opt = Some(Init);

        'main_loop: loop {
            while let Some(msg) = message_opt {
                match msg {
                    Exit => {
                        info!("Exiting application");
                        shutdown_tx.send(())?;
                        break 'main_loop;
                    }
                    _ => message_opt = app.update(&msg).await
                }
            }

            terminal.draw(|frame| app.draw(frame))?;

            message_opt = ui_rx.recv().await;
        }

        Ok(())
    }

    async fn update(&mut self, action: &Action) -> Option<Action> {
        debug!("action: {:?}", action);
        self.widgets.iter_mut().for_each(|(k, v)| {
            v.update(action);
        });
        match action {
            ReceivedInput(event) => match self.handle_key_event(event) {
                Ok(Some(m)) => Some(m),
                Ok(None) => None,
                Err(e) => {
                    self.status_widget.info = e.to_string();
                    None
                }
            },

            SubsFetched(subtitles) => {
                self.search_widget.update_subtitles(&subtitles);
                self.status_widget.info = format!("{} results", subtitles.data.len());

                self.status_widget.in_progress = false;

                None
            }

            LanguagesUpdated => {
                self.config_provider.modify(|c: &Config| {
                    let mut updated = c.clone();
                    updated.languages = self.languages_widget.languages().clone();
                    updated
                });

                let languages = self.languages_widget.languages();
                let query: String = self.search_widget.query();

                Some(Tuple(
                    Box::from(SwitchScreen(Search)),
                    Box::from(FetchSubs(query, languages)),
                ))
            }

            UserLoggedIn(user_info) => {
                self.user_widget.requests = user_info.data.downloads_count;
                self.user_widget.remaining = user_info.data.remaining_downloads;


                Some(Tuple(
                    Box::from(SwitchScreen(Search)),
                    Box::from(ChangeStatus(format!(
                        "Logged in as {}",
                        user_info.data.username
                    ))),
                ))
            }

            UserLoggedOut => {
                self.user_widget.requests = 0;
                self.user_widget.remaining = 0;

                None
            }

            SearchQueryUpdated => {
                self.status_widget.in_progress = true;

                let languages = self.languages_widget.languages();
                let query = self.search_widget.query();

                Some(FetchSubs(query, languages))
            }

            FetchSubs(query, languages) => {
                self.status_widget.in_progress = true;

                self.subtitles_tx
                    .send(SubtitlesQuery {
                        query: query.to_string(),
                        languages: languages.to_vec(),
                        id: None,
                    })
                    .await;

                None
            }

            Init => {
                // self.account_widget.refresh();

                let query: String = self.search_widget.query();
                if !query.is_empty() {
                    let languages = self.languages_widget.languages();
                    Some(FetchSubs(query, languages))
                } else {
                    None
                }
            }

            DownloadedSubs(downloaded) => {
                self.status_widget.info = format!("Downloaded: {:?}", downloaded.path);
                self.user_widget.requests = downloaded.requests;
                self.user_widget.remaining = downloaded.remaining;

                None
            }

            SwitchScreen(screen) => {
                self.current_screen = *screen;

                None
            }

            Exit => None,

            EnabledLimitSubsToId(id) => {
                let languages = self.languages_widget.languages();
                let query = self.search_widget.query();
                self.subtitles_tx
                    .send(SubtitlesQuery {
                        query,
                        languages,
                        id: Some(*id),
                    })
                    .await
                    .unwrap(); // todo

                None
            }

            ChangeStatus(status) => {
                self.status_widget.info = status.clone();

                None
            }

            FeatureInfo(id) => None,

            Tuple(action1, action2) => {
                if let Some(a1) = Box::pin(self.update(action1)).await {
                    Box::pin(self.update(&a1)).await
                } else {
                    if let Some(a2) = Box::pin(self.update(action2)).await {
                        Box::pin(self.update(&a2)).await
                    } else {
                        None
                    }
                }
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
        self.widgets.get_mut("nav").unwrap().render(frame, content[2]);
        // self.nav_widget.render(frame, content[2]);

        match &self.current_screen {
            Search => {
                self.search_widget.render(frame, content[0]);
            }
            Language => {
                self.languages_widget.render(frame, content[0]);
            }
            Account => {
                self.widgets.get_mut("account").unwrap().render(frame, content[0]);
                // self.account_widget.render(frame, content[0]);
            }
            About => {
                self.about_widget.render(frame, content[0]);
            }
        }
    }

    fn handle_key_event(&mut self, event: &Event) -> Result<Option<Action>> {
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
                    Search => Ok(self.search_widget.handle_key_event(event)),
                    Language => Ok(self.languages_widget.handle_key_event(event)),
                    Account => {
                        let aw = self.widgets.get_mut("account").unwrap();
                        Ok(aw.handle_key_event(event))
                    },
                    About => Ok(self.about_widget.handle_key_event(event)),
                },
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Default, Hash, Eq, PartialEq, Copy, Clone)]
pub enum CurrentScreen {
    #[default]
    Search,
    Account,
    Language,
    About,
}
