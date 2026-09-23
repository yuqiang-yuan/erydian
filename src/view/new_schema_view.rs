use std::collections::BTreeMap;

use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, base::{Selectable, StyledExt, input::InputState}, component::{
        ActiveTheme, Icon, WindowExt, button::{Button, ButtonGroup, ButtonVariants}, checkbox::Checkbox, input::{Input, Textarea}, notification::NotificationType, select::Select,
    }, div, prelude::FluentBuilder,
};

use crate::{actions::SchemaCreatedAction, dialect::DialectKind, model::SchemaDocument, view::AttrState};

pub struct NewSchemaView {
    dialect: DialectKind,
    name_state: Entity<InputState>,
    attr_states: Vec<AttrState>,
}

impl NewSchemaView {
    pub fn new(dialect: DialectKind, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            dialect,
            name_state: cx
                .new(|cx| InputState::new(window, cx).validate(|s, _| !s.trim().is_empty())),
            attr_states: dialect
                .database_attributes()
                .iter()
                .map(|a| AttrState::from_kind(&a.kind, window, cx))
                .collect::<Vec<_>>(),
        }
    }

    fn prepare_attr_states(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.attr_states = self
            .dialect
            .database_attributes()
            .iter()
            .map(|a| AttrState::from_kind(&a.kind, window, cx))
            .collect::<Vec<_>>()
    }
}

impl Render for NewSchemaView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .pt_8()
            .v_flex()
            .gap_4()
            .child(div().text_xl().text_center().child("New Schema"))
            .child(
                div().h_flex().justify_center().child(
                    ButtonGroup::new("database-button-group")
                        .multiple(false)
                        .child(
                            Button::new("mysql-databse-button")
                                .when(self.dialect == DialectKind::MySql, |this| this.success())
                                .selected(self.dialect == DialectKind::MySql)
                                .icon(Icon::default().path("icons/mysql-logo.svg"))
                                .label("MySQL")
                        )
                        .child(
                            Button::new("postgresql-database-button")
                                .when(self.dialect == DialectKind::PostgreSql, |this| {
                                    this.success()
                                })
                                .selected(self.dialect == DialectKind::PostgreSql)
                                .icon(Icon::default().path("icons/postgresql-logo.svg"))
                                .label("PostgreSQL")
                        )
                        .on_click(cx.listener(|this, selected_indexes: &Vec<usize>, window, cx| {
                            if selected_indexes.is_empty() {
                                return;
                            }

                            let target_kind = match selected_indexes[0] {
                                0 => DialectKind::MySql,
                                1 => DialectKind::PostgreSql,
                                _ => return,
                            };

                            if target_kind == this.dialect {
                                return;
                            }

                            this.dialect = target_kind;

                            this.prepare_attr_states(window, cx);

                            cx.notify();
                        }))
                )
            )
            .child(
                div()
                    .w_128()
                    .self_center()
                    .bg(cx.theme().secondary)
                    .px_4()
                    .pt_4()
                    .pb_5()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .v_flex()
                    .gap_4()
                    .child(
                        div()
                            .v_flex()
                            .gap_1()
                            .child(div().pl_2().child("Name"))
                            .child(Input::new(&self.name_state)),
                    )
                    .children(
                        self.dialect
                            .database_attributes()
                            .iter()
                            .enumerate()
                            .map(|(i, attr)| {
                                let opt_state = &self.attr_states.get(i);

                                if opt_state.is_none() {
                                    println!("can not find state for item: {i}");
                                    return div().into_any_element();
                                }

                                div()
                                    .v_flex()
                                    .gap_1()
                                    .child(div().pl_2().child(attr.label.to_string()))
                                    .child(if let Some(state) = opt_state {
                                        match state {
                                            AttrState::Text(entity) => {
                                                Input::new(entity).into_any_element()
                                            },

                                            AttrState::MultilineText(entity) => {
                                                Textarea::new(entity).into_any_element()
                                            },

                                            AttrState::Select(entity) => {
                                                Select::new(entity).into_any_element()
                                            },

                                            AttrState::Bool(b) => Checkbox::new(format!(
                                                "new-{}-{}",
                                                self.dialect.name(),
                                                i
                                            ))
                                            .checked(*b)
                                            .on_click(cx.listener(move |this, v: &bool, _, cx| {
                                                this.attr_states[i] = AttrState::Bool(*v);
                                                cx.notify();
                                            }))
                                            .into_any_element(),
                                        }
                                    } else {
                                        div().into_any_element()
                                    })
                                    .into_any_element()
                            })
                            .collect::<Vec<_>>(),
                    ),
            )
            .child(
                div()
                    .mt_4()
                    .h_flex()
                    .justify_center()
                    .gap_4()
                    .child(
                        Button::new("new-schema-confirmed-button")
                            .primary()
                            .label("Create")
                            .on_click(cx.listener(|this, _, window, cx| {
                                if this.name_state.read(cx).value().is_empty() {
                                    window.push_notification((NotificationType::Error, "Name is required"), cx);
                                    return;
                                }

                                let db_attrs = this.dialect
                                    .database_attributes()
                                    .iter()
                                    .zip(&this.attr_states)
                                    .map(|(a, s)| {
                                        (a.key.to_string(), s.value(cx))
                                    })
                                    .collect::<BTreeMap<_, _>>();

                                let doc = SchemaDocument::new(this.dialect, this.name_state.read(cx).value().to_string(), db_attrs);
                                if let Ok(s) = serde_json::to_string(&doc) {
                                    window.dispatch_action(Box::new(SchemaCreatedAction {schema_json: s}), cx);
                                } else {
                                    window.push_notification((NotificationType::Error, "Handle new schema failed"), cx);
                                    return;
                                }
                            }))
                    )
                    .child(
                        Button::new("new-schema-cancel-button")
                            .label("Cancel")
                    )
            )
    }
}
