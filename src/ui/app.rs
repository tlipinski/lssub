use crate::config::{Config, ConfigProvider};
use crate::osb::get_download_link::get_download_link;
use crate::osb::login::login;
use crate::osb::user_info;
use crate::osb::user_info::{UserInfo, get_user_info};
use crate::secret::{clear, retrieve, store};
use crate::ui::about_widget::AboutWidget;
use crate::ui::account_widget::AccountWidget;
use crate::ui::action_handler::Component;
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

pub struct App {
    current_screen: CurrentScreen,
    subtitles_tx: Sender<SubtitlesQuery>,
    config_provider: ConfigProvider,
    modal_visible: bool,
    widgets: HashMap<String, Box<dyn Component>>,
    query: String,
    languages: Vec<String>,
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

        let mut components: HashMap<String, Box<dyn Component>> = HashMap::new();
        components.insert(
            "account".into(),
            Box::new(AccountWidget::new(task_runner.clone())),
        );
        components.insert("nav".into(), Box::new(NavWidget::new()));
        components.insert("user".into(), Box::new(UserWidget::from()));
        components.insert(
            "search".into(),
            Box::new(SearchWidget::from(
                base_path,
                file_name,
                task_runner.clone(),
            )?),
        );
        components.insert(
            "languages".into(),
            Box::new(LanguagesWidget::new(
                config_provider.get_config()?.languages,
            )),
        );
        components.insert(
            "status".into(),
            Box::new(StatusWidget::from(spinner.clone())),
        );
        components.insert("about".into(), Box::new(AboutWidget::new()));

        let mut app = App {
            current_screen: CurrentScreen::default(),
            config_provider,
            subtitles_tx,
            modal_visible: false,
            widgets: components,
            query: "".to_string(),
            languages: vec![],
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
                    _ => message_opt = app.update(&msg).await,
                }
            }

            terminal.draw(|frame| app.draw(frame))?;

            message_opt = ui_rx.recv().await;
        }

        Ok(())
    }

    async fn update(&mut self, action: &Action) -> Option<Action> {
        debug!("action: {:?}", action);
        let map = self.widgets.values_mut().map(|(component)| {
            component.update(action);
        });
        match action {
            ReceivedInput(event) => match self.handle_key_event(event) {
                Ok(Some(m)) => Some(m),
                Ok(None) => None,
                Err(e) => Some(ChangeStatus(e.to_string())),
            },

            SubsFetched(subtitles) => {
                Some(ChangeStatus(format!("{} results", subtitles.data.len())))
            }

            LanguagesUpdated(languages) => {
                self.languages = languages.clone();
                
                self.config_provider.modify(|c: &Config| {
                    let mut updated = c.clone();
                    updated.languages = self.languages.clone();
                    updated
                });

                Some(Tuple(
                    Box::from(SwitchScreen(Search)),
                    Box::from(FetchSubs(self.query.clone(), self.languages.clone())),
                ))
            }

            UserLoggedIn(user_info) => Some(Tuple(
                Box::from(SwitchScreen(Search)),
                Box::from(ChangeStatus(format!(
                    "Logged in as {}",
                    user_info.data.username
                ))),
            )),

            UserLoggedOut => None,

            SearchQueryUpdated(query) => {
                // self.status_widget.in_progress = true;
                self.query = query.clone();
                Some(FetchSubs(query.clone(), self.languages.clone()))
            }

            FetchSubs(query, languages) => {
                // self.status_widget.in_progress = true;

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
                let query: String = self.query.clone();
                if !query.is_empty() {
                    let languages = self.languages.clone();
                    Some(FetchSubs(query, languages))
                } else {
                    None
                }
            }

            DownloadedSubs(downloaded) => {
                Some(ChangeStatus(format!("Downloaded: {:?}", downloaded.path)))
            }

            SwitchScreen(screen) => {
                self.current_screen = *screen;

                None
            }

            Exit => None,

            EnabledLimitSubsToId(id) => {
                let languages = self.languages.clone();
                let query = self.query.clone();
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

            _ => None,
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

        self.widgets
            .get_mut("status")
            .unwrap()
            .render(frame, status[0]);
        self.widgets
            .get_mut("user")
            .unwrap()
            .render(frame, status[1]);
        self.widgets
            .get_mut("nav")
            .unwrap()
            .render(frame, content[2]);

        match &self.current_screen {
            Search => {
                self.widgets
                    .get_mut("search")
                    .unwrap()
                    .render(frame, content[0]);
            }
            Language => {
                self.widgets
                    .get_mut("languages")
                    .unwrap()
                    .render(frame, content[0]);
            }
            Account => {
                self.widgets
                    .get_mut("account")
                    .unwrap()
                    .render(frame, content[0]);
            }
            About => {
                self.widgets
                    .get_mut("account")
                    .unwrap()
                    .render(frame, content[0]);
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
                            // self.search_widget.help = false;
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
                        // self.search_widget.help = !self.search_widget.help;
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
                    Search => Ok(self
                        .widgets
                        .get_mut("search")
                        .unwrap()
                        .handle_key_event(event)),
                    Language => Ok(self
                        .widgets
                        .get_mut("language")
                        .unwrap()
                        .handle_key_event(event)),
                    Account => Ok(self
                        .widgets
                        .get_mut("account")
                        .unwrap()
                        .handle_key_event(event)),
                    About => Ok(self
                        .widgets
                        .get_mut("about")
                        .unwrap()
                        .handle_key_event(event)),
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
