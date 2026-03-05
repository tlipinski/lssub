use crate::config::{Config, ConfigProvider};
use crate::ui::actions::Action;
use crate::ui::actions::Action::{FetchSubs, LanguagesUpdated};
use crate::ui::app::CurrentScreen::Main;
use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

pub struct LanguagesScreen {
    config_provider: ConfigProvider,
    input: Input,
}

impl LanguagesScreen {
    pub fn new(config_provider: ConfigProvider) -> anyhow::Result<LanguagesScreen> {
        let languages = config_provider.get_config()?.languages;
        Ok(Self {
            config_provider,
            input: Input::new(languages.join(",")),
        })
    }

    async fn update(&mut self, action: Action) -> Result<Vec<Action>> {
        match action {
            _ => Ok(vec![]),
        }
    }

    pub fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => {
                    self.config_provider.modify(|c: &Config| {
                        let mut updated = c.clone();
                        updated.languages = self.languages().clone();
                        updated
                    });
                    Ok(Some(LanguagesUpdated))
                }

                _ => {
                    self.input.handle_event(&event);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn languages(&self) -> Vec<String> {
        let langs: String = self.input.value().into();
        let v = langs.split(",").collect::<Vec<&str>>();
        v.iter().map(|&x| String::from(x)).collect::<Vec<String>>()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut title = "Language".to_string().bold();

        let block = Block::bordered().title(title).border_set(border::THICK);

        let par = Line::from(self.input.value().bold());

        let view = Paragraph::new(par).block(block);

        let x = self.input.visual_cursor();
        frame.set_cursor_position((area.x + (x + 1) as u16, area.y + 1));

        frame.render_widget(view, area);
    }
}
