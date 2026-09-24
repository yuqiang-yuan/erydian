mod detail_view;
mod diagram_view;
mod editor_view;
mod main_view;
mod new_schema_view;
mod welcome_view;

use std::collections::BTreeMap;

pub use editor_view::EditorView;
use gpui_kit::{
    AnyElement, App, AppContext, Entity, IntoElement, SharedString, Window, base::input::{InputState, TextareaState}, component::{
        checkbox::Checkbox, input::{Input, Textarea}, select::{SearchableVec, Select, SelectGroup, SelectItem, SelectState},
    },
};
pub use main_view::MainView;
pub use new_schema_view::NewSchemaView;
pub use welcome_view::WelcomeView;

use crate::{
    dialect::{
        Dialect,
        DialectKind::{self},
        MySqlDialect, PostgreSqlDialect,
    },
    model::{AttrKind, AttrSpec, AttrValue, ColumnTypeSpec},
};

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

/// options to control UI component or it's state
#[derive(Debug, Clone, Default)]
pub struct AttrFieldOptions {
    pub id: Option<String>,
}

impl AttrFieldOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// To store dynamic components' state
#[derive(Debug, Clone)]
pub enum AttrField {
    Text(Entity<InputState>),
    MultilineText(Entity<TextareaState>),
    Select(Entity<SelectState<SearchableVec<SharedString>>>),
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

    pub fn set_value(&mut self, value: AttrValue, window: &mut Window, cx: &mut App) {
        match self {
            AttrField::Text(entity) => {
                if let AttrValue::Text(text) = value {
                    entity.update(cx, |this, cx| {
                        this.set_value(text.unwrap_or(String::new()).to_string(), window, cx)
                    });
                } else {
                    entity.update(cx, |this, cx| this.set_value(String::new(), window, cx));
                }
            }

            AttrField::MultilineText(entity) => {
                if let AttrValue::Text(text) = value {
                    entity.update(cx, |this, cx| {
                        this.set_value(text.unwrap_or(String::new()).to_string(), window, cx)
                    });
                } else {
                    entity.update(cx, |this, cx| this.set_value(String::new(), window, cx));
                }
            }

            AttrField::Select(entity) => {
                if let AttrValue::Text(text) = value {
                    entity.update(cx, |this, cx| {
                        this.set_selected_value(&SharedString::from(text.unwrap_or(String::new())), window, cx)
                    });
                } else {
                    entity.update(cx, |this, cx| {
                        this.set_selected_value(&SharedString::default(), window, cx)
                    });
                }
            }

            AttrField::Bool(entity) => {
                if let AttrValue::Bool(b) = value {
                    entity.update(cx, |this, _| *this = b);
                } else {
                    entity.update(cx, |this, _| *this = false);
                }
            }
        }
    }

    /// Clear the entity of state's value
    pub fn clear_value(&mut self, window: &mut Window, cx: &mut App) {
        match self {
            AttrField::Text(entity) => entity.update(cx, |this, cx| this.set_value("", window, cx)),
            AttrField::MultilineText(entity) => {
                entity.update(cx, |this, cx| this.set_value("", window, cx))
            }
            AttrField::Select(entity) => entity.update(cx, |this, cx| {
                this.set_selected_value(&SharedString::default(), window, cx)
            }),
            AttrField::Bool(entity) => entity.update(cx, |this, _| *this = false),
        }
    }

    pub fn from_kind(kind: &AttrKind, window: &mut Window, cx: &mut App) -> AttrField {
        match kind {
            AttrKind::Text {
                default,
                multiple_line: false,
            } => AttrField::Text(cx.new(|cx| {
                InputState::new(window, cx).default_value(default.clone().unwrap_or(String::new()))
            })),

            AttrKind::Text {
                default,
                multiple_line: true,
            } => AttrField::MultilineText(cx.new(|cx| {
                TextareaState::new(window, cx)
                    .auto_grow(1, 5)
                    .default_value(default.clone().unwrap_or(String::new()).to_string())
            })),

            AttrKind::Select { options } => {
                let items = SearchableVec::new(options.clone());
                let state = cx.new(|cx| SelectState::new(items, None, window, cx).searchable(true));
                AttrField::Select(state)
            }

            AttrKind::Bool => AttrField::Bool(cx.new(|_| false)),
        }
    }

    /// comp_id 是针对无状态的组件来设计的，比如 Checkbox、Switch 等。
    pub fn render_component(&self, options: Option<AttrFieldOptions>, cx: &App) -> AnyElement {
        match self {
            AttrField::Text(entity) => Input::new(entity).into_any_element(),
            AttrField::MultilineText(entity) => Textarea::new(entity).into_any_element(),
            AttrField::Select(entity) => Select::new(entity).cleanable(true).into_any_element(),
            AttrField::Bool(entity) => {
                let comp_id = if let Some(opt) = &options
                    && let Some(id) = &opt.id
                {
                    id.clone()
                } else {
                    "check-box".to_string()
                };

                let entity = entity.clone();
                Checkbox::new(comp_id)
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

/// Simplified data type select
impl gpui_kit::component::searchable_list::SearchableListItem for ColumnTypeSpec {
    type Value = Self;

    fn title(&self) -> gpui_kit::SharedString {
        self.name.into()
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

// some helper methods related to UI
impl DialectKind {
    // build grouped column type data for select state
    pub fn grouped_column_type_spec(&self) -> SearchableVec<SelectGroup<ColumnTypeSpec>> {
        let items = match self {
            DialectKind::MySql => MySqlDialect::column_types(),
            DialectKind::PostgreSql => PostgreSqlDialect::column_types(),
        };

        let mut item_map = BTreeMap::<String, Vec<ColumnTypeSpec>>::new();
        items.into_iter().for_each(|t| {
            item_map
                .entry(format!("{}", t.category))
                .or_default()
                .push(t)
        });

        let mut grouped_items = SearchableVec::new(vec![]);
        item_map.into_iter().for_each(|(category, items)| {
            grouped_items.push(SelectGroup::new(&category).items(items));
        });

        grouped_items
    }
}
