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
    LanguagesUpdated, Multiple, SearchQueryUpdated, SwitchScreen, UserLoggedIn, UserLoggedOut,
};
use crate::ui::app::Action::{ReceivedInput, SubsFetched};
use crate::ui::app::Screen::{About, Account, Language, Search};
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
    modal_visible: bool,
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
        let mut widget_actions = self
            .widgets
            .values_mut()
            .map(|(component)| component.update(action))
            .collect::<Vec<Option<Action>>>();

        let new_action = match action {
            ReceivedInput(event) => self.handle_key_event(event),

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

                Some(Multiple(vec![
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
                self.active_screen = *screen;

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

            Multiple(actions) => {
                let mut next_action = None;
                for action in actions {
                    if let Some(a) = Box::pin(self.update(action)).await {
                        next_action = Box::pin(self.update(&a)).await;
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
            _ => Some(Multiple(actions)),
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

    fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        if (self.modal_visible) {
            if let Event::Key(key_event) = event {
                match key_event {
                    KeyEvent {
                        code: KeyCode::F(1),
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Esc, ..
                    } => match self.active_screen {
                        Search => {
                            // self.search_widget.help = false;
                            self.modal_visible = false;
                            None
                        }
                        _ => None,
                    },
                    _ => None,
                }
            } else {
                None
            }
        } else if let Event::Key(key_event) = event {
            match key_event {
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => Some(SwitchScreen(Search)),
                KeyEvent {
                    code: KeyCode::F(1),
                    ..
                } => match self.active_screen {
                    Search => {
                        // self.search_widget.help = !self.search_widget.help;
                        self.modal_visible = true;
                        None
                    }
                    _ => None,
                },
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

                _ => match self.active_screen {
                    Search => self
                        .widgets
                        .get_mut(&WidgetName::Search)
                        .unwrap()
                        .handle_key_event(event),
                    Language => self
                        .widgets
                        .get_mut(&WidgetName::Languages)
                        .unwrap()
                        .handle_key_event(event),
                    Account => self
                        .widgets
                        .get_mut(&WidgetName::Account)
                        .unwrap()
                        .handle_key_event(event),
                    About => self
                        .widgets
                        .get_mut(&WidgetName::About)
                        .unwrap()
                        .handle_key_event(event),
                },
            }
        } else {
            None
        }
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
