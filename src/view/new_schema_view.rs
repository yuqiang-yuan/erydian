use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    base::{StyledExt, input::InputState},
    component::{
        ActiveTheme, Icon,
        button::{Button, ButtonVariants},
        checkbox::Checkbox,
        input::Input,
        select::Select,
    },
    div,
    prelude::FluentBuilder,
};

use crate::{dialect::DialectKind, model::AttrState};

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
                .map(|a| a.kind.state(window, cx))
                .collect::<Vec<_>>(),
        }
    }

    fn prepare_attr_states(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.attr_states = self
            .dialect
            .database_attributes()
            .iter()
            .map(|a| a.kind.state(window, cx))
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
                div()
                    .w_full()
                    .h_flex()
                    .gap_4()
                    .justify_center()
                    .items_center()
                    .child(
                        Button::new("new-mysql-button")
                            .when(self.dialect == DialectKind::MySql, |this| this.success())
                            .icon(Icon::default().path("mysql-logo.svg"))
                            .label("MySQL")
                            .on_click(cx.listener(|this, _, window, cx| {
                                if this.dialect == DialectKind::MySql {
                                    return;
                                }

                                this.dialect = DialectKind::MySql;
                                this.prepare_attr_states(window, cx);

                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("new-postgresql-button")
                            .when(self.dialect == DialectKind::PostgreSql, |this| {
                                this.success()
                            })
                            .icon(Icon::default().path("postgresql-logo.svg"))
                            .label("PostgreSQL")
                            .on_click(cx.listener(|this, _, window, cx| {
                                if this.dialect == DialectKind::PostgreSql {
                                    return;
                                }
                                this.dialect = DialectKind::PostgreSql;
                                this.prepare_attr_states(window, cx);

                                cx.notify();
                            })),
                    ),
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
                                            }
                                            AttrState::Select(entity) => {
                                                Select::new(entity).into_any_element()
                                            }
                                            AttrState::Bool(b) => Checkbox::new(format!(
                                                "new-{}-{}",
                                                self.dialect.name(),
                                                i
                                            ))
                                            .checked(*b)
                                            .on_click(cx.listener(move |this, v: &bool, _, cx| {
                                                this.attr_states[i] = AttrState::Bool(!*v);
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
    }
}
