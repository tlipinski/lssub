use std::path::PathBuf;
use crate::ui::app::CurrentScreen;
use osb::subtitles::SubtitlesResponse;
use ratatui::crossterm::event::Event;
use osb::login::Credentials;
use crate::ui::downloader::Downloaded;
use crate::ui::subtitles_fetcher::SubtitlesQuery;

pub enum Action {
    Input(Event),
    SubsFetched(SubtitlesResponse),
    SpinnerUpdate(char),
    LanguagesUpdated,
    LoggedIn,
    LoggedOut,
    QueryUpdated(String),
    FetchSubs(String, Vec<String>),
    StartSpinner,
    StopSpinner,
    Init,
    DownloadSubs(i64, String),
    DownloadedSubs(Downloaded),
    SwitchScreen(CurrentScreen),
    DownloadSubsFailed(String),
    EnabledLimitSubsToId(i64),
    DisabledLimitSubsToId,
    Exit,
}
