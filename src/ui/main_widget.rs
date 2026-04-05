use crate::config::{Config, ConfigProvider};
use crate::osb::osb_client::OsbClient;
use crate::osb::subtitles::{SubtitlesRequest, subtitles};
use crate::ui::about_widget::AboutWidget;
use crate::ui::account_widget::AccountWidget;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    Exit, FetchSubtitles, LanguagesInitialized, LanguagesUpdated, Multi, SearchParamsInitialized,
    SearchParamsUpdated, SearchQueryInitialized, SearchQueryUpdated, SubtitlesFetched,
    SwitchScreen, Tick,
};
use crate::ui::component::Component;
use crate::ui::languages_widget::LanguagesWidget;
use crate::ui::main_widget::Screen::{About, Account, Language, Search};
use crate::ui::nav_widget::NavWidget;
use crate::ui::search_widget::SearchWidget;
use crate::ui::spinner::Spinner;
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_list_widget::QueryParams;
use crate::ui::task_runner::{Task, TaskRunner};
use crate::ui::user_widget::UserWidget;
use Action::RunTask;
use KeyCode::{Char, Esc, F};
use anyhow::Error;
use log::{debug, error, info};
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Sender;

pub struct MainWidget {
    active_screen: Screen,
    debouncer_tx: Sender<()>,
    config_provider: ConfigProvider,
    widgets: HashMap<WidgetName, Box<dyn Component>>,
    task_runner: TaskRunner,
    query_snapshot: Option<String>,
    params_snapshot: Option<QueryParams>,
    languages_snapshot: Option<Vec<String>>,
}

impl Component for MainWidget {
    fn update(&mut self, action: &Action) -> Option<Action> {
        match action {
            Tick | Multi(_) => {}
            _ => debug!("--- {:?}", action),
        }

        let mut widget_actions = self
            .widgets
            .values_mut()
            .map(|component| component.update(action))
            .collect::<Vec<Option<Action>>>();

        let new_action = match action {
            LanguagesUpdated(languages) => {
                self.languages_snapshot = Some(languages.clone());

                let _ = self.config_provider.modify(|c: &Config| {
                    let mut updated = c.clone();
                    updated.languages.clone_from(&Some(languages.clone()));
                    updated
                });

                Some(Multi(vec![SwitchScreen(Search), FetchSubtitles]))
            }

            SearchQueryInitialized(query) => {
                self.query_snapshot = Some(query.clone());
                Some(FetchSubtitles)
            }

            SearchParamsInitialized(params) => {
                self.params_snapshot = Some(params.clone());
                Some(FetchSubtitles)
            }

            LanguagesInitialized(languages) => {
                self.languages_snapshot = Some(languages.clone());
                Some(FetchSubtitles)
            }

            SearchQueryUpdated(query) => {
                self.query_snapshot = Some(query.clone());
                let debouncer = self.debouncer_tx.clone();
                tokio::spawn(async move {
                    debouncer.send(()).await.expect("Sending to channel failed");
                });
                None
            }

            SearchParamsUpdated(params) => {
                self.params_snapshot = Some(params.clone());
                Some(FetchSubtitles)
            }

            FetchSubtitles => match (
                self.query_snapshot.clone(),
                self.params_snapshot.clone(),
                self.languages_snapshot.clone(),
            ) {
                (Some(query), Some(params), Some(languages)) => {
                    info!(
                        "Fetching subtitles for query: {query:?}, languages: {languages:?}, params: {params:?}"
                    );
                    let request = Self::subtitles_request(query, params, languages);
                    let fetch_subs_task = Task::new("fetch subs", async move {
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

                    Some(RunTask(fetch_subs_task))
                }

                _ => {
                    // not initialized yet
                    None
                }
            },

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

    fn render(&mut self, frame: &mut Frame, area: Rect) {
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
}

impl MainWidget {
    pub fn new(
        base_path: &Path,
        file_name: Option<&str>,
        debouncer_tx: Sender<()>,
        task_runner: TaskRunner,
        config_provider: ConfigProvider,
        spinner: Arc<RwLock<Spinner>>,
    ) -> MainWidget {
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
        components.insert(WidgetName::Status, Box::new(StatusWidget::from(spinner)));

        MainWidget {
            active_screen: Screen::default(),
            config_provider,
            debouncer_tx,
            widgets: components,
            task_runner,
            query_snapshot: None,
            params_snapshot: None,
            languages_snapshot: None,
        }
    }

    fn subtitles_request(
        query: String,
        params: QueryParams,
        languages: Vec<String>,
    ) -> SubtitlesRequest {
        SubtitlesRequest {
            query,
            languages,
            id: params.feature_id,
            parent_id: params.parent_feature_id,
            ai_translated: if params.exclude_ai {
                "exclude".to_string()
            } else {
                "include".to_string()
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::osb::subtitles::{Attributes, FeatureDetails, Subtitle, Uploader};
    use crate::osb::user_info::User;
    use crate::ui::actions::Action::UserLoggedIn;
    use crossterm::event::Event::Key;
    use crossterm::event::KeyCode::Tab;
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};
    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    struct TestTerminal(Terminal<TestBackend>);

    impl Default for TestTerminal {
        fn default() -> Self {
            TestTerminal(Terminal::new(TestBackend::new(100, 20)).unwrap())
        }
    }

    impl Default for MainWidget {
        fn default() -> Self {
            let (ui_tx, _) = tokio::sync::mpsc::channel::<Action>(100);
            let (debouncer_tx, _) = tokio::sync::mpsc::channel::<()>(100);
            let spinner = Arc::new(RwLock::new(Spinner { c: ' ' }));

            MainWidget::new(
                Path::new("."),
                None,
                debouncer_tx,
                TaskRunner::new(ui_tx),
                ConfigProvider::default(),
                spinner,
            )
        }
    }

    #[test]
    fn main_screen() {
        let mut app = MainWidget::default();

        let mut terminal = TestTerminal::default().0;
        terminal
            .draw(|frame| app.render(frame, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn main_screen_help() {
        let mut app = MainWidget::default();

        input_key(&mut app, F(1), KeyModifiers::NONE);

        let mut terminal = TestTerminal::default().0;
        terminal
            .draw(|frame| app.render(frame, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn account_screen() {
        let mut app = MainWidget::default();

        app.update(&SwitchScreen(Account));

        input_text(&mut app, "test_user");
        input_key(&mut app, Tab, KeyModifiers::NONE);
        input_text(&mut app, "test_pass");

        let mut terminal = TestTerminal::default().0;
        terminal
            .draw(|frame| app.render(frame, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn main_screen_logged_in() {
        let mut app = MainWidget::default();

        let user = User {
            username: "user".to_string(),
            downloads_count: 4,
            remaining_downloads: 6,
            level: "vip".to_string(),
            allowed_translations: 10,
            allowed_downloads: 10,
        };
        app.update(&UserLoggedIn(user));

        let mut terminal = TestTerminal::default().0;
        terminal
            .draw(|frame| app.render(frame, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn account_screen_logged_in() {
        let mut app = MainWidget::default();

        let user = User {
            username: "user".to_string(),
            downloads_count: 4,
            remaining_downloads: 6,
            level: "vip".to_string(),
            allowed_translations: 10,
            allowed_downloads: 10,
        };

        app.update(&UserLoggedIn(user));
        app.update(&SwitchScreen(Account));

        let mut terminal = TestTerminal::default().0;
        terminal
            .draw(|frame| app.render(frame, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn ai_excluded_main_screen() {
        let mut app = MainWidget::default();

        input_key(&mut app, Char('t'), KeyModifiers::CONTROL);

        let mut terminal = TestTerminal::default().0;
        terminal
            .draw(|frame| app.render(frame, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn subs_query() {
        let mut app = MainWidget::default();

        let subs = vec![Subtitle {
            id: "1234".to_string(),
            r#type: String::new(),
            attributes: Attributes {
                feature_details: FeatureDetails {
                    feature_id: 0,
                    title: "title".to_string(),
                    movie_name: "movie name".to_string(),
                    year: Some(2004),
                    parent_feature_id: None,
                    parent_title: None,
                },
                language: "en".to_string(),
                download_count: 4,
                new_download_count: 8,
                ai_translated: true,
                votes: 8,
                upload_date: "2024-04-24T10:10:10".to_string(),
                release: "release".to_string(),
                files: vec![],
                uploader: Uploader {
                    name: "".to_string(),
                    rank: "".to_string(),
                },
            },
        }];
        app.update(&SubtitlesFetched(subs));
        input_text(&mut app, "title");
        input_key(&mut app, KeyCode::Down, KeyModifiers::NONE);

        let mut terminal = TestTerminal::default().0;
        terminal
            .draw(|frame| app.render(frame, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn subs_fetched() {
        let mut app = MainWidget::default();

        let subs = vec![
            Subtitle {
                id: "1234".to_string(),
                r#type: String::new(),
                attributes: Attributes {
                    feature_details: FeatureDetails {
                        feature_id: 0,
                        title: "title".to_string(),
                        movie_name: "movie name".to_string(),
                        year: Some(2004),
                        parent_feature_id: None,
                        parent_title: None,
                    },
                    language: "en".to_string(),
                    download_count: 4,
                    new_download_count: 8,
                    ai_translated: true,
                    votes: 8,
                    upload_date: "2024-04-24T10:10:10".to_string(),
                    release: "release".to_string(),
                    files: vec![],
                    uploader: Uploader {
                        name: "uploader".into(),
                        rank: String::new(),
                    },
                },
            },
            Subtitle {
                id: "1243".to_string(),
                r#type: String::new(),
                attributes: Attributes {
                    feature_details: FeatureDetails {
                        feature_id: 0,
                        title: "title 2".to_string(),
                        movie_name: "movie name 2".to_string(),
                        year: None,
                        parent_feature_id: None,
                        parent_title: None,
                    },
                    language: "pl".to_string(),
                    download_count: 8,
                    new_download_count: 14,
                    ai_translated: false,
                    votes: 0,
                    upload_date: "2014-04-24T10:10:10".to_string(),
                    release: "release 2".to_string(),
                    files: vec![],
                    uploader: Uploader {
                        name: "uploader".to_string(),
                        rank: "".to_string(),
                    },
                },
            },
        ];
        app.update(&SubtitlesFetched(subs));

        let mut terminal = TestTerminal::default().0;
        terminal
            .draw(|frame| app.render(frame, frame.area()))
            .unwrap();

        assert_snapshot!(terminal.backend());
    }

    fn input_text(app: &mut MainWidget, text: &str) {
        text.chars().for_each(|c| {
            app.handle_key_event(&Event::Key(KeyEvent {
                code: Char(c),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }));
        });
    }

    fn input_key(app: &mut MainWidget, code: KeyCode, modifiers: KeyModifiers) {
        app.handle_key_event(&Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }));
    }
}
