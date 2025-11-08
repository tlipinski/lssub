use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug)]
pub struct LanguageWidget {
    pub input: Input,
    pub active: bool,
}

impl LanguageWidget {
    pub fn from() -> Self {
        LanguageWidget {
            input: Input::new("pl".into()),
            active: false,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut title = " Language ".to_string().bold();

        let block = Block::bordered()
            .title(title)
            .border_set(border::THICK);

        let par = Line::from(self.input.value().bold());

        let view = Paragraph::new(par).block(block);

        let x = self.input.visual_cursor();
        frame.set_cursor_position((area.x + (x + 1) as u16, area.y + 1));

        frame.render_widget(view, area);
    }

    pub fn handle_key_event(&mut self, event: Event) {
        self.input.handle_event(&event);
    }
}
