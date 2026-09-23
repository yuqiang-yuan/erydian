mod detail_view;
mod diagram_view;
mod editor_view;
mod main_view;
mod new_schema_view;
mod welcome_view;

pub use editor_view::EditorView;
use gpui_kit::{
    AnyElement, App, AppContext, Entity, IntoElement, Window, base::input::{InputState, TextareaState}, component::{checkbox::Checkbox, input::{Input, Textarea}, select::{SearchableVec, Select, SelectState}},
};
pub use main_view::MainView;
pub use new_schema_view::NewSchemaView;
pub use welcome_view::WelcomeView;

use crate::model::{AttrKind, AttrSpec, AttrValue};

#[derive(Debug, Clone)]
pub struct AttrState {
    pub attr: AttrSpec,
    pub field: AttrField,
}

impl AttrState {
    pub fn from_attr_spec(attr: &AttrSpec, window: &mut Window, cx: &mut App) -> Self {
        Self {
            attr: attr.clone(),
            field: AttrField::from_kind(&attr.kind, window, cx),
        }
    }

    pub fn value_entry(&self, cx: &App) -> (String, AttrValue) {
        (self.attr.key.to_string(), self.field.value(cx))
    }
}

/// To store dynamic components' state
#[derive(Debug, Clone)]
pub enum AttrField {
    Text(Entity<InputState>),
    MultilineText(Entity<TextareaState>),
    Select(Entity<SelectState<SearchableVec<&'static str>>>),
    Bool(Entity<bool>),
}

impl AttrField {
    pub fn value(&self, cx: &App) -> AttrValue {
        match self {
            AttrField::Text(state) => AttrValue::Text(Some(state.read(cx).value().to_string())),
            AttrField::MultilineText(state) => {
                AttrValue::Text(Some(state.read(cx).value().to_string()))
            }
            AttrField::Select(state) => {
                AttrValue::Text(state.read(cx).selected_value().map(|s| s.to_string()))
            }
            AttrField::Bool(v) => AttrValue::Bool(*v.read(cx)),
        }
    }

    pub fn from_kind(kind: &AttrKind, window: &mut Window, cx: &mut App) -> AttrField {
        match kind {
            AttrKind::Text {
                default,
                multiple_line: false,
            } => AttrField::Text(cx.new(|cx| {
                InputState::new(window, cx).default_value(default.clone().unwrap_or("").to_string())
            })),

            AttrKind::Text {
                default,
                multiple_line: true,
            } => AttrField::MultilineText(cx.new(|cx| {
                TextareaState::new(window, cx)
                    .rows(5)
                    .default_value(default.clone().unwrap_or("").to_string())
            })),

            AttrKind::Select { options } => {
                let items = SearchableVec::new(options.iter().map(|s| *s).collect::<Vec<_>>());
                let state = cx.new(|cx| SelectState::new(items, None, window, cx).searchable(true));
                AttrField::Select(state)
            }

            AttrKind::Bool => AttrField::Bool(cx.new(|_| false)),
        }
    }

    /// comp_id 是针对无状态的组件来设计的，比如 Checkbox、Switch 等。
    pub fn render_component(&self, comp_id: &str, cx: &App) -> AnyElement {
        match self {
            AttrField::Text(entity) => Input::new(entity).into_any_element(),
            AttrField::MultilineText(entity) => Textarea::new(entity).into_any_element(),
            AttrField::Select(entity) => Select::new(entity).into_any_element(),
            AttrField::Bool(entity) => {
                let entity = entity.clone();
                Checkbox::new(comp_id.to_string())
                    .checked(*entity.read(cx))
                    .on_click(move |v, _, cx| {
                        entity.update(cx, |val, cx| {
                            *val = *v;
                            cx.notify();
                        });
                    })
                    .into_any_element()
            }
        }
    }
}

/// What is selected
#[derive(Debug, Clone, PartialEq)]
pub enum SelectedItem {
    Table(String),
    Relationship(String),
}
