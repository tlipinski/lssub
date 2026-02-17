use crate::ui::app::CurrentScreen;
use osb::subtitles::SubtitlesResponse;
use ratatui::crossterm::event::Event;

#[derive(Debug)]
pub enum UiMessage {
    Input(Event),
    SubsFetched(SubtitlesResponse),
    SpinnerUpdate(char),
    LanguagesUpdated(Vec<String>),
    QueryUpdated(String),
    FetchSubs(String, Vec<String>),
    StartSpinner,
    StopSpinner,
    Init,
    DownloadSubs(i64),
    SwitchScreen(CurrentScreen),
    Exit,
    // Tuple(Box<UiMessage>, Box<UiMessage>),
}
