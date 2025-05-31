use ratatui::crossterm::event::KeyEvent;
use osb::subtitles::SubtitlesResponse;

#[derive(Debug)]
pub enum UiEvent {
    Input(KeyEvent),
    ResultsUpdate(SubtitlesResponse),
}

