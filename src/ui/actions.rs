use std::path::PathBuf;
use crate::ui::app::CurrentScreen;
use osb::subtitles::SubtitlesResponse;
use ratatui::crossterm::event::Event;
use osb::login::Credentials;
use crate::ui::downloader::Downloaded;
use crate::ui::subtitles_fetcher::SubtitlesQuery;

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
