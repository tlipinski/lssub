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
    LoginFailed(String),
    LoginSuccessful,
    QueryUpdated(String),
    FetchSubs(String, Vec<String>),
    StartSpinner,
    StopSpinner,
    Init,
    DownloadSubs(i64),
    DownloadedSubs(PathBuf),
    SwitchScreen(CurrentScreen),
    DownloadSubsFailed(String),
    Exit,
}
