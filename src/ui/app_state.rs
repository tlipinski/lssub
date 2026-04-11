use crate::ui::subs_list_widget::QueryParams;

#[derive(Default, Clone, Debug)]
pub struct AppState {
    pub query: String,
    pub params: QueryParams,
    pub languages: Vec<String>,
}
