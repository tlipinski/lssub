use crate::ui::actions::Action;
use crate::ui::app_state::AppState;
use crossterm::event::Event;
use ratatui::Frame;
use ratatui::layout::Rect;

pub trait Component {
    fn update(&mut self, action: &Action, state: AppState) -> Option<(Action, AppState)>;
    fn handle_key_event(&mut self, event: &Event, state: AppState) -> Option<(Action, AppState)>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
