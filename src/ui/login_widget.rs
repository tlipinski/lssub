use crate::ui::ui_messages::UiMessage;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug)]
pub struct LoginWidget {
    pub username: Input,
    pub password: Input,
}

impl LoginWidget {
    pub fn from() -> Self {
        LoginWidget {
            username: Input::new("".into()),
            password: Input::new("".into()),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut title = " Login ".to_string().bold();

        let block = Block::bordered().title(title).border_set(border::THICK);

        let par = Line::from(self.username.value().bold());

        let view = Paragraph::new(par).block(block);

        let x = self.username.visual_cursor();
        frame.set_cursor_position((area.x + (x + 1) as u16, area.y + 1));

        frame.render_widget(view, area);
    }

    pub fn handle_key_event(&mut self, event: Event) -> Option<UiMessage> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => Some(UiMessage::LoggedIn),
                _ => {
                    self.username.handle_event(&event);
                    None
                }
            }
        } else {
            None
        }
    }
}
