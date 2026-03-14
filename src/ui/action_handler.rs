use crate::ui::actions::Action;
use crossterm::event::Event;

pub trait ActionHandler {
    fn update(&mut self, action: &Action) -> ();
    fn handle_key_event(&mut self, event: &Event) -> Option<Action>;
}
