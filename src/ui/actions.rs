use crate::osb::languages::Language;
use crate::osb::subtitles::Subtitle;
use crate::osb::user_info::User;
use crate::ui::downloader::Downloaded;
use crate::ui::main_widget::Screen;
use crate::ui::subs_list_widget::QueryParams;
use crate::ui::task_runner::Task;

#[derive(Debug)]
pub enum Action {
    Init,
    Exit,
    Tick,
    NoOp,
    Multi(Vec<Action>),

    SwitchScreen(Screen),

    SearchQueryInitialized(String),
    SearchQueryUpdated,
    SearchParamsInitialized(QueryParams),
    SearchParamsUpdated,

    FetchSubtitles,
    SubtitlesFetched(Vec<Subtitle>),
    SubtitleDownloaded(Downloaded),

    LanguagesUpdated,
    LanguagesInitialized(Vec<String>),
    LanguagesFetched(Vec<Language>),
    UserLanguagesFetched(Vec<String>),

    UserLoggedIn(User),
    UserLoggedOut,

    ChangeStatus(String),

    RunTask(Task),
    StartProgress,
    StopProgress,
}
