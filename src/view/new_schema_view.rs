use std::collections::BTreeMap;

use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, base::{Selectable, StyledExt, input::InputState}, component::{
        ActiveTheme, Icon, WindowExt, button::{Button, ButtonGroup, ButtonVariants}, input::Input, notification::NotificationType,
    }, div, prelude::FluentBuilder,
};

use crate::{actions::SchemaCreatedAction, dialect::DialectKind, model::SchemaDocument, view::{AttrFieldOptions, AttrState}};

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
                .map(|a| AttrState::from_attr_spec(a, window, cx))
                .collect::<Vec<_>>(),
        }
    }

    /// recalculate the states while switching database type
    fn prepare_attr_states(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.attr_states = self
            .dialect
            .database_attributes()
            .iter()
            .map(|a| AttrState::from_attr_spec(a, window, cx))
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
                        self.attr_states.iter().enumerate().map(|(i, attr_state)| {
                            div()
                                .v_flex()
                                .gap_1()
                                .child(div().pl_2().child(attr_state.attr.label.to_string()))
                                .child(attr_state.field.render_component(Some(AttrFieldOptions::new().id(format!("new-{}-{}", self.dialect.name(), i))), cx))
                                .into_any_element()
                        })
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

                                let db_attrs = this.attr_states
                                    .iter()
                                    .map(|attr_state| attr_state.value_entry(cx))
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
