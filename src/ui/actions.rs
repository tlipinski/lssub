use crate::osb::login::Credentials;
use crate::osb::subtitles::SubtitlesResponse;
use crate::ui::app::CurrentScreen;
use crate::ui::downloader::Downloaded;
use crate::ui::subtitles_fetcher::SubtitlesQuery;
use ratatui::crossterm::event::Event;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Action {
    ReceivedInput(Event),
    SubsFetched(SubtitlesResponse),
    LanguagesUpdated,
    UserLoggedIn,
    UserLoggedOut,
    SearchQueryUpdated,
    FetchSubs(String, Vec<String>),
    Init,
    DownloadedSubs(Downloaded),
    SwitchScreen(CurrentScreen),
    EnabledLimitSubsToId(i64),
    ChangeStatus(String),
    Exit,
}
