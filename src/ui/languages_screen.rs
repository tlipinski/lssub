use crate::config::{Config, ConfigProvider};
use crate::ui::actions::Action;
use crate::ui::actions::Action::{FetchSubs, LanguagesUpdated};
use crate::ui::app::CurrentScreen::Main;
use crate::ui::language_widget::LanguageWidget;
use ratatui::Frame;

pub struct LanguagesScreen {
    config_provider: ConfigProvider,
    pub language_widget: LanguageWidget,
}

impl LanguagesScreen {
    pub fn new(config_provider: ConfigProvider) -> anyhow::Result<LanguagesScreen> {
        let languages = config_provider.get_config()?.languages;
        Ok(Self {
            config_provider,
            language_widget: LanguageWidget::from(languages),
        })
    }

    async fn update(&mut self, action: Action) -> anyhow::Result<Vec<Action>> {
        match action {
            LanguagesUpdated(languages) => {
                // self.current_screen = Main;
                // let query: String = self.search_widget.input.value().into();
                self.config_provider.modify(|c: &Config| {
                    let mut updated = c.clone();
                    updated.languages = languages.clone();
                    updated
                });
                Ok(vec![FetchSubs(query, languages)])
            }

            _ => Ok(vec![]),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        todo!()
    }
}
