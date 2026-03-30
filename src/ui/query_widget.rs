use ratatui::buffer::Buffer;
use crate::ui::pad::BlockTitlePadExt;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph, Widget};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
use crate::ui::actions::Action;
use crate::ui::actions::Action::SearchQueryUpdated;

#[derive(Debug)]
pub struct QueryWidget {
    input: Input,
}

impl Widget for &QueryWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized
    {
        let block = Block::bordered()
            .title_pad("Search")
            .border_set(border::PLAIN);

        let par = Line::from(self.input.value().bold());

        let view = Paragraph::new(par).block(block);

        view.render(area, buf);
    }
}

impl QueryWidget {
    pub fn from(search_text: String) -> Self {
        QueryWidget {
            input: Input::from(search_text),
        }
    }

    pub fn query(&self) -> String {
        self.input.value().into()
    }

    pub fn visual_cursor(&self) -> usize {
        self.input.visual_cursor()
    }

    pub fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        if let Some(state_changed) = self.input.handle_event(event)
            && state_changed.value
        {
            Some(SearchQueryUpdated(self.input.value().into()))
        } else {
            None
        }
    }
}
