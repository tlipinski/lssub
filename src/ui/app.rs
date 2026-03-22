use crate::config::{Config, ConfigProvider};
use crate::osb::get_download_link::get_download_link;
use crate::osb::login::login;
use crate::osb::osb_client::OsbClient;
use crate::osb::subtitles::{SubtitlesRequest, subtitles};
use crate::ui::about_widget::AboutWidget;
use crate::ui::account_widget::AccountWidget;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    ChangeStatus, Exit, FeatureInfo, FetchSubtitles, Init, LanguagesAndConfigFetched,
    LanguagesFetched, LanguagesUpdated, Multi, SearchQueryUpdated, StartProgress, StopProgress,
    SubtitleDownloaded, SwitchScreen, Tick, UserLoggedIn, UserLoggedOut,
};
use crate::ui::app::Action::{InputReceived, SubtitlesFetched};
use crate::ui::app::Screen::{About, Account, Language, Search};
use crate::ui::component::Component;
use crate::ui::debouncer::debouncer_task;
use crate::ui::downloader::Downloader;
use crate::ui::input_handler::handle_input_task;
use crate::ui::languages_widget::LanguagesWidget;
use crate::ui::logged_in_widget::LoggedInWidget;
use crate::ui::login_widget::LoginWidget;
use crate::ui::nav_widget::NavWidget;
use crate::ui::query_widget::QueryWidget;
use crate::ui::search_widget::{SearchWidget, SubtitlesQuery};
use crate::ui::spinner::{Spinner, spinner_task};
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_list_widget::SubsListWidget;
use crate::ui::task_runner::{Task, TaskRunner};
use crate::ui::user_widget::UserWidget;
use Action::RunTask;
use KeyCode::{Char, Esc, F};
use anyhow::{Error, Result, bail};
use clap::builder::TypedValueParser;
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
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

pub struct App {
    active_screen: Screen,
    debouncer_tx: Sender<SubtitlesQuery>,
    ui_rx: Receiver<Action>,
    config_provider: ConfigProvider,
    widgets: HashMap<WidgetName, Box<dyn Component>>,
    task_runner: TaskRunner,
    query: SubtitlesQuery,
    languages: Vec<String>,
    initialized: bool,
}

impl App {
    pub async fn new(
        base_path: &Path,
        file_name: Option<&str>,
    ) -> Result<App> {
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::channel::<Action>(100);
        let (debouncer_tx, debouncer_rx) = tokio::sync::mpsc::channel::<SubtitlesQuery>(100);

        let task_runner = TaskRunner::new(ui_tx.clone());

        let spinner = Arc::new(RwLock::new(Spinner { c: ' ' }));
        let spinner_clone = spinner.clone();

        tokio::spawn(handle_input_task(ui_tx.clone()));
        tokio::spawn(debouncer_task(debouncer_rx, ui_tx.clone()));
        tokio::spawn(spinner_task(spinner_clone));

        let config_provider = ConfigProvider::default();

        let mut components: HashMap<WidgetName, Box<dyn Component>> = HashMap::new();
        components.insert(WidgetName::Nav, Box::new(NavWidget::new()));
        components.insert(WidgetName::User, Box::new(UserWidget::from()));
        components.insert(WidgetName::About, Box::new(AboutWidget::new()));
        components.insert(WidgetName::Account, Box::new(AccountWidget::new()));
        components.insert(
            WidgetName::Search,
            Box::new(SearchWidget::from(base_path, file_name)),
        );
        components.insert(WidgetName::Languages, Box::new(LanguagesWidget::new()));
        components.insert(
            WidgetName::Status,
            Box::new(StatusWidget::from(spinner.clone())),
        );

        Ok(App {
            active_screen: Screen::default(),
            config_provider,
            debouncer_tx,
            ui_rx,
            widgets: components,
            task_runner,
            query: SubtitlesQuery {
                query: file_name.unwrap_or("").into(),
                ..SubtitlesQuery::default()
            },
            languages: vec![],
            initialized: false,
        })
    }

    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<()> {
        let mut message_opt = Some(Init);

        'main_loop: loop {
            while let Some(msg) = message_opt {
                match msg {
                    Exit => {
                        info!("Exiting application");
                        break 'main_loop;
                    }
                    _ => message_opt = self.update(&msg),
                }
            }

            terminal.draw(|frame| self.draw(frame))?;

            message_opt = self.ui_rx.recv().await;
        }

        Ok(())
    }

    fn update(&mut self, action: &Action) -> Option<Action> {
        match action {
            Tick | Multi(_) | InputReceived(_) => {}
            _ => debug!("--- {:?}", action),
        }

        let mut widget_actions = self
            .widgets
            .values_mut()
            .map(|(component)| component.update(action))
            .collect::<Vec<Option<Action>>>();

        let new_action = match action {
            InputReceived(event) => self.handle_key_event(event),

            LanguagesUpdated(languages) => {
                self.languages.clone_from(languages);

                self.config_provider.modify(|c: &Config| {
                    let mut updated = c.clone();
                    updated.languages.clone_from(&self.languages);
                    updated
                });

                Some(Multi(vec![SwitchScreen(Search), FetchSubtitles]))
            }

            SearchQueryUpdated(query) => {
                if (self.initialized) {
                    if self.query.query == query.query {
                        self.query = query.clone();

                        Some(FetchSubtitles)
                    } else {
                        self.query = query.clone();
                        let q = query.clone();
                        let debouncer = self.debouncer_tx.clone();
                        tokio::spawn(async move {
                            debouncer.send(q).await;
                        });
                        None
                    }
                } else {
                    self.query = query.clone();
                    None
                }
            }

            FetchSubtitles => {
                let request = self.subtitles_request();
                let task = Task::new("fetch subs", async move {
                    if request.query.len() < 3 {
                        Ok(SubtitlesFetched(vec![]))
                    } else {
                        let result = subtitles(OsbClient::default(), request).await;
                        match result {
                            Ok(subtitles) => Ok(SubtitlesFetched(subtitles)),
                            Err(e) => {
                                error!("Error fetching subtitles {e}");
                                Err(Error::msg("Error fetching subtitles list, check logs"))
                            }
                        }
                    }
                });

                Some(RunTask(task))
            }

            LanguagesFetched(languages) => {
                let user_languages = self.config_provider.get_config().unwrap().languages;
                self.initialized = true; // todo rework it

                Some(LanguagesAndConfigFetched(languages.clone(), user_languages))
            }

            SwitchScreen(screen) => {
                self.active_screen = *screen;
                None
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

            RunTask(task) => {
                self.task_runner.run(task.clone());
                None
            }

            _ => None,
        };

        if let Some(action) = new_action {
            widget_actions.push(Some(action));
        }

        let actions: Vec<Action> = widget_actions.into_iter().flatten().collect();

        match actions.len() {
            0 => None,
            1 => actions.into_iter().next(),
            _ => Some(Multi(actions)),
        }
    }

    fn subtitles_request(&self) -> SubtitlesRequest {
        SubtitlesRequest {
            query: self.query.query.clone(),
            id: self.query.params.feature_id,
            parent_id: self.query.params.parent_feature_id,
            languages: self.languages.clone(),
            ai_translated: if (self.query.params.exclude_ai) {
                "exclude".to_string()
            } else {
                "include".to_string()
            },
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

        self.active_widget().render(frame, content[0]);
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
                match (key_event.code, key_event.modifiers) {
                    (Esc | F(2), KeyModifiers::NONE) => Some(SwitchScreen(Search)),
                    (F(3), KeyModifiers::NONE) => Some(SwitchScreen(Account)),
                    (F(4), KeyModifiers::NONE) => Some(SwitchScreen(Language)),
                    (F(10), KeyModifiers::NONE) | (Char('c'), KeyModifiers::CONTROL) => Some(Exit),
                    (F(12), KeyModifiers::NONE) => Some(SwitchScreen(About)),
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
