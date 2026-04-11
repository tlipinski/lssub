use crate::ui::subs_list_widget::QueryParams;

#[derive(Default, Clone, Debug)]
pub struct AppState {
    pub query: Option<String>,
    pub params: Option<QueryParams>,
    pub languages: Option<Vec<String>>,
}
