use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window, base::{StyledExt, input::InputState}, component::{
        ActiveTheme, WindowExt, button::{Button, ButtonVariants}, input::Input, notification::NotificationType, searchable_list::SearchableListItem, select::{Select, SelectDelegate, SelectState },
    }, div,
};

use crate::{
    actions::SchemaCreatedAction,
    model::{DbKind, SchemaDocument},
};

pub struct NewSchemaView {
    kind: DbKind,
    name_state: Entity<InputState>,
    kind_state: Entity<SelectState<Vec<DbKind>>>,
}

impl SearchableListItem for DbKind {
    type Value = Self;

    fn title(&self) -> gpui_kit::SharedString {
        SharedString::from(format!("{}", self))
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

impl NewSchemaView {
    pub fn new(kind: DbKind, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            kind,
            name_state: cx
                .new(|cx| InputState::new(window, cx).validate(|s, _| !s.trim().is_empty())),
            kind_state: cx.new(|cx| {
                SelectState::new(
                    vec![
                        DbKind::MySql,
                        DbKind::PostgreSql,
                    ],
                    None,
                    window,
                    cx,
                )
            }),
        }
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
                            .child(div().pl_2().child("Database type"))
                            .child(Select::new(&self.kind_state)),
                    )
                    .child(
                        div()
                            .v_flex()
                            .gap_1()
                            .child(div().pl_2().child("Name"))
                            .child(Input::new(&self.name_state)),
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
                                let kind = this.kind_state.read(cx).selected_value();

                                if kind.is_none() {
                                    window.push_notification(
                                        (NotificationType::Error, "Database type is required"),
                                        cx,
                                    );
                                    return;
                                }

                                if this.name_state.read(cx).value().is_empty() {
                                    window.push_notification(
                                        (NotificationType::Error, "Name is required"),
                                        cx,
                                    );
                                    return;
                                }

                                let doc = SchemaDocument::new(
                                    *kind.unwrap(),
                                    this.name_state.read(cx).value().to_string(),
                                );
                                if let Ok(s) = serde_json::to_string(&doc) {
                                    window.dispatch_action(
                                        Box::new(SchemaCreatedAction { schema_json: s }),
                                        cx,
                                    );
                                } else {
                                    window.push_notification(
                                        (NotificationType::Error, "Handle new schema failed"),
                                        cx,
                                    );
                                    return;
                                }
                            })),
                    )
                    .child(Button::new("new-schema-cancel-button").label("Cancel")),
            )
    }
}
