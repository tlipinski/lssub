use crate::ui::actions::Action;
use crossterm::event::Event;
use ratatui::Frame;
use ratatui::layout::Rect;

pub trait Component {
    fn update(&mut self, action: &Action) -> Option<Action>;
    fn handle_key_event(&mut self, event: &Event) -> Option<Action>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
