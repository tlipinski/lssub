use std::path::PathBuf;
use crate::ui::app::CurrentScreen;
use osb::subtitles::SubtitlesResponse;
use ratatui::crossterm::event::Event;
use osb::login::Credentials;
use crate::ui::downloader::Downloaded;
use crate::ui::subtitles_fetcher::SubtitlesQuery;

pub enum UiMessage {
    Input(Event),
    SubsFetched(SubtitlesResponse),
    SpinnerUpdate(char),
    LanguagesUpdated(Vec<String>),
    Login(Credentials),
    LoginFailed(String),
    UpdateUser,
    Logout,
    GoToLogin,
    QueryUpdated(String),
    FetchSubs(String, Vec<String>),
    StartSpinner,
    StopSpinner,
    Init,
    DownloadSubs(i64),
    DownloadedSubs(Downloaded),
    UpdateDownloadCount(i32, i32),
    UpdateUsername(String),
    SwitchScreen(CurrentScreen),
    DownloadSubsFailed(String),
    LimitSubsToId(i64),
    NoLimitSubsToId,
    Exit,
}
