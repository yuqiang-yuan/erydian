mod schema;

use gpui_kit::{
    App, AppContext, Entity, Window,
    base::input::InputState,
    component::select::{SearchableVec, SelectState},
};
pub use schema::SchemaDocument;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrKind {
    Text { default: Option<String> },
    Select { options: Vec<String> },
    Bool,
}

impl AttrKind {
    pub fn state(&self, window: &mut Window, cx: &mut App) -> AttrState {
        match self {
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

pub struct AttrSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: AttrKind,
    pub required: bool,
}

/// To store dynamic components' state
pub enum AttrState {
    Text(Entity<InputState>),
    Select(Entity<SelectState<SearchableVec<String>>>),
    Bool(bool),
}
