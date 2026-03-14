use crate::config::{Config, ConfigProvider};
use crate::osb::get_download_link::get_download_link;
use crate::osb::login::login;
use crate::osb::user_info;
use crate::osb::user_info::{UserInfo, get_user_info};
use crate::secret::{clear, retrieve, store};
use crate::ui::about_widget::AboutWidget;
use crate::ui::account_widget::AccountWidget;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, DisabledLimitSubsToId, DownloadedSubs, EnabledLimitSubsToId, Exit, FeatureInfo, FetchSubs, Init, LanguagesUpdated, Multi, SearchQueryUpdated, StartProgress, StopProgress, SwitchScreen, UserLoggedIn, UserLoggedOut};
use crate::ui::app::Action::{ReceivedInput, SubsFetched};
use crate::ui::app::Screen::{About, Account, Language, Search};
use crate::ui::component::Component;
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
    active_screen: Screen,
    subtitles_tx: Sender<SubtitlesQuery>,
    config_provider: ConfigProvider,
    widgets: HashMap<WidgetName, Box<dyn Component>>,
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

        let mut components: HashMap<WidgetName, Box<dyn Component>> = HashMap::new();
        components.insert(WidgetName::Nav, Box::new(NavWidget::new()));
        components.insert(WidgetName::User, Box::new(UserWidget::from()));
        components.insert(WidgetName::About, Box::new(AboutWidget::new()));
        components.insert(
            WidgetName::Account,
            Box::new(AccountWidget::new(task_runner.clone())),
        );
        components.insert(
            WidgetName::Search,
            Box::new(SearchWidget::from(
                base_path,
                file_name,
                task_runner.clone(),
            )),
        );
        components.insert(
            WidgetName::Languages,
            Box::new(LanguagesWidget::new(
                config_provider.get_config()?.languages,
            )),
        );
        components.insert(
            WidgetName::Status,
            Box::new(StatusWidget::from(spinner.clone())),
        );

        let mut app = App {
            active_screen: Screen::default(),
            config_provider,
            subtitles_tx,
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
                    _ => message_opt = app.update(&msg),
                }
            }

            terminal.draw(|frame| app.draw(frame))?;

            message_opt = ui_rx.recv().await;
        }

        Ok(())
    }

    fn update(&mut self, action: &Action) -> Option<Action> {
        debug!("action: {:?}", action);
        let mut widget_actions = self
            .widgets
            .values_mut()
            .map(|(component)| component.update(action))
            .collect::<Vec<Option<Action>>>();

        let new_action = match action {
            ReceivedInput(event) => self.handle_key_event(event),

            Init => {
                let query: String = self.query.clone();
                if !query.is_empty() {
                    let languages = self.languages.clone();
                    Some(FetchSubs(query, languages))
                } else {
                    None
                }
            }

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

                Some(Multi(vec![
                    SwitchScreen(Search),
                    FetchSubs(self.query.clone(), self.languages.clone()),
                ]))
            }

            UserLoggedIn(user_info) => Some(ChangeStatus(format!(
                "Logged in as {}",
                user_info.data.username
            ))),

            UserLoggedOut => Some(ChangeStatus("Logged out".to_string())),

            SearchQueryUpdated(query) => {
                self.query = query.clone();
                Some(FetchSubs(query.clone(), self.languages.clone()))
            }

            FetchSubs(query, languages) => {
                let subtitles_tx = self.subtitles_tx.clone();
                let query = query.to_string();
                let languages = languages.to_vec();

                tokio::spawn(async move {
                    subtitles_tx
                        .send(SubtitlesQuery {
                            query,
                            languages,
                            id: None,
                        })
                        .await;
                });

                None
            }


            DownloadedSubs(downloaded) => {
                Some(ChangeStatus(format!("Downloaded: {:?}", downloaded.path)))
            }

            SwitchScreen(screen) => {
                self.active_screen = *screen;
                None
            }

            Exit => None,

            EnabledLimitSubsToId(id) => {
                let languages = self.languages.clone();
                let query = self.query.clone();
                let id = *id;
                let subtitles_tx = self.subtitles_tx.clone();

                tokio::spawn(async move {
                    subtitles_tx
                        .send(SubtitlesQuery {
                            query,
                            languages,
                            id: Some(id),
                        })
                        .await
                });

                None
            }

            DisabledLimitSubsToId => {
                Some(FetchSubs(self.query.clone(), self.languages.clone()))
            }

            Multi(actions) => {
                let mut next_action = None;
                for action in actions {
                    if let Some(a) = self.update(action) {
                        next_action = self.update(&a);
                    }
                }

                next_action
            }

            _ => None,
        };

        if let Some(action) = new_action {
            widget_actions.push(Some(action));
        }

        let actions: Vec<Action> = widget_actions.into_iter().filter_map(|a| a).collect();

        match actions.len() {
            0 => None,
            1 => actions.into_iter().next(),
            _ => Some(Multi(actions)),
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
            .get_mut(&WidgetName::Status)
            .unwrap()
            .render(frame, status[0]);
        self.widgets
            .get_mut(&WidgetName::User)
            .unwrap()
            .render(frame, status[1]);
        self.widgets
            .get_mut(&WidgetName::Nav)
            .unwrap()
            .render(frame, content[2]);

        match &self.active_screen {
            Search => {
                self.widgets
                    .get_mut(&WidgetName::Search)
                    .unwrap()
                    .render(frame, content[0]);
            }
            Language => {
                self.widgets
                    .get_mut(&WidgetName::Languages)
                    .unwrap()
                    .render(frame, content[0]);
            }
            Account => {
                self.widgets
                    .get_mut(&WidgetName::Account)
                    .unwrap()
                    .render(frame, content[0]);
            }
            About => {
                self.widgets
                    .get_mut(&WidgetName::About)
                    .unwrap()
                    .render(frame, content[0]);
            }
        }
    }

    fn active_widget(&mut self) -> &mut Box<dyn Component> {
        let widget_name = match self.active_screen {
            Search => WidgetName::Search,
            Account => WidgetName::Account,
            Language => WidgetName::Languages,
            About => WidgetName::About,
        };
        self.widgets.get_mut(&widget_name).unwrap()
    }

    fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        self.active_widget().handle_key_event(event).or_else(|| {
            if let Event::Key(key_event) = event {
                match key_event {
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => Some(SwitchScreen(Search)),
                    KeyEvent {
                        code: KeyCode::F(2),
                        ..
                    } => Some(SwitchScreen(Search)),
                    KeyEvent {
                        code: KeyCode::F(3),
                        ..
                    } => Some(SwitchScreen(Account)),
                    KeyEvent {
                        code: KeyCode::F(4),
                        ..
                    } => Some(SwitchScreen(Language)),
                    KeyEvent {
                        code: KeyCode::F(10),
                        ..
                    } => Some(Exit),
                    KeyEvent {
                        code: KeyCode::F(12),
                        ..
                    } => Some(SwitchScreen(About)),

                    _ => None,
                }
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
enum WidgetName {
    Account,
    Nav,
    User,
    Search,
    Languages,
    Status,
    About,
}

#[derive(Debug, Default, Hash, Eq, PartialEq, Copy, Clone)]
pub enum Screen {
    #[default]
    Search,
    Account,
    Language,
    About,
}
