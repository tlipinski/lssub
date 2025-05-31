use osb::subtitles::SubtitlesResponse;
use ratatui::crossterm::event::KeyEvent;

#[derive(Debug)]
pub enum UiEvent {
    Input(KeyEvent),
    ResultsUpdate(SubtitlesResponse),
}

