use crate::ui::actions::Action;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug)]
pub struct LanguageWidget {
    pub input: Input,
}

impl LanguageWidget {
    pub fn from(languages: Vec<String>) -> Self {
        LanguageWidget {
            input: Input::new(languages.join(",")),
        }
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

    pub fn languages(&self) -> Vec<String> {
        let langs: String = self.input.value().into();
        let v = langs.split(",").collect::<Vec<&str>>();
        v.iter().map(|&x| String::from(x)).collect::<Vec<String>>()
    }

    pub fn handle_key_event(&mut self, event: Event) -> Option<Action> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => Some(Action::LanguagesUpdated(self.languages())),
                _ => {
                    self.input.handle_event(&event);
                    None
                }
            }
        } else {
            None
        }
    }
}
