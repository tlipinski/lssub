use osb::subtitles::SubtitlesResponse;
use ratatui::crossterm::event::Event;
use crate::ui::app::CurrentScreen;

#[derive(Debug)]
pub enum UiEvent {
    Input(Event),
    ResultsUpdate(SubtitlesResponse),
    SpinnerUpdate(char),
    LanguagesUpdated(Vec<String>),
    QueryUpdated(String),
    FetchSubs(String, Vec<String>),
    StartSpinner,
    StopSpinner,
    Init,
    DownloadConfirmed(i64),
    SwitchScreen(CurrentScreen),
    Tuple(Box<UiEvent>, Box<UiEvent>),
    Exit
}

