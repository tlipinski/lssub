use crate::ui::subs_list_widget::QueryParams;

#[derive(Default, Clone, Debug)]
pub struct AppState {
    pub query_snapshot: Option<String>,
    pub params_snapshot: Option<QueryParams>,
    pub languages_snapshot: Option<Vec<String>>,
}
