use crate::ui::actions::Action;
use crate::ui::actions::Action::SearchQueryUpdated;
use crate::ui::app_state::AppState;
use crate::ui::component::Component;
use crate::ui::pad::BlockTitlePadExt;
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph, Widget};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug, Default)]
pub struct QueryWidget {
    input: Input,
}

impl Component for QueryWidget {
    fn update(&mut self, action: &Action, state: AppState) -> Option<(Action, AppState)> {
        match action {
            Action::Init => {
                self.input = Input::from(state.query);
                None
            }
            _ => None,
        }
    }

    fn handle_key_event(
        &mut self,
        event: &Event,
        app_state: AppState,
    ) -> Option<(Action, AppState)> {
        if let Some(state_changed) = self.input.handle_event(event)
            && state_changed.value
        {
            let new_state = AppState {
                query: self.input.value().into(),
                ..app_state
            };
            Some((SearchQueryUpdated, new_state))
        } else {
            None
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title_pad("Search")
            .border_set(border::PLAIN);

        let par = Line::from(self.input.value().bold());

        let view = Paragraph::new(par).block(block);

        frame.set_cursor_position((area.x + (self.input.visual_cursor() + 1) as u16, area.y + 1));

        view.render(area, frame.buffer_mut());
    }
}
