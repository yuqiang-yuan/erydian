mod main_view;
mod new_schema_view;
mod welcome_view;
mod editor_view;
mod diagram_view;
mod detail_view;

use gpui_kit::{
    App, AppContext, Entity, Window,
    base::input::InputState,
    component::select::{SearchableVec, SelectState},
};
pub use main_view::MainView;
pub use new_schema_view::NewSchemaView;
pub use welcome_view::WelcomeView;
pub use editor_view::EditorView;

use crate::model::{AttrKind, AttrValue};

/// To store dynamic components' state
#[derive(Debug, Clone)]
pub enum AttrState {
    Text(Entity<InputState>),
    Select(Entity<SelectState<SearchableVec<String>>>),
    Bool(bool),
}

impl AttrState {
    pub fn value(&self, cx: &App) -> AttrValue {
        match self {
            AttrState::Text(state) => AttrValue::Text(Some(state.read(cx).value().to_string())),
            AttrState::Select(state) => {
                AttrValue::Text(state.read(cx).selected_value().cloned())
            }
            AttrState::Bool(v) => AttrValue::Bool(*v),
        }
    }

    pub fn from_kind(kind: &AttrKind, window: &mut Window, cx: &mut App) -> AttrState {
        match kind {
            AttrKind::Text { default } => AttrState::Text(cx.new(|cx| {
                InputState::new(window, cx).default_value(default.clone().unwrap_or(String::new()))
            })),
            AttrKind::Select { options } => {
                let items = SearchableVec::new(options.clone());
                let state = cx.new(|cx| SelectState::new(items, None, window, cx).searchable(true));
                AttrState::Select(state)
            }
            AttrKind::Bool => AttrState::Bool(false),
        }
    }
}

/// What is selected
#[derive(Debug, Clone)]
pub enum SelectedItem {
    Table(String),
    Relationship(String),
}
