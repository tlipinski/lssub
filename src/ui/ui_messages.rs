use std::path::PathBuf;
use crate::ui::app::CurrentScreen;
use osb::subtitles::SubtitlesResponse;
use ratatui::crossterm::event::Event;
use osb::login::Credentials;

#[derive(Debug)]
pub enum UiMessage {
    Input(Event),
    SubsFetched(SubtitlesResponse),
    SpinnerUpdate(char),
    LanguagesUpdated(Vec<String>),
    Login(Credentials),
    QueryUpdated(String),
    FetchSubs(String, Vec<String>),
    StartSpinner,
    StopSpinner,
    Init,
    DownloadSubs(i64),
    DownloadedSubs(PathBuf),
    SwitchScreen(CurrentScreen),
    StatusError(String),
    Exit,
}
