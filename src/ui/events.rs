use osb::subtitles::SubtitlesResponse;
use ratatui::crossterm::event::Event;

#[derive(Debug)]
pub enum UiEvent {
    Input(Event),
    ResultsUpdate(SubtitlesResponse),
    SpinnerUpdate(char),
    LanguagesUpdated(Vec<String>),
    QueryUpdated(String),
    FetchSubs(String, Vec<String>)
}

