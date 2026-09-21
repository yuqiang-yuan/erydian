use std::rc::Rc;

use crate::{dialect::DialectKind, model::SchemaDocument, settings::AppSettings};
use gpui_kit::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    base::{StyledExt, h_resizable, resizable_panel, v_resizable},
    component::{ActiveTheme, Icon, scroll::ScrollableElement},
    div, px,
};

pub struct EditorView {
    schema: Entity<SchemaDocument>,
}

impl EditorView {
    pub fn new(schema: SchemaDocument, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            schema: cx.new(|_| schema),
        }
    }

    fn object_list(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .p_1()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .w_full()
                    .h_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        div().p_1().rounded_sm().bg(cx.theme().list_active).child(
                            Icon::default()
                                .path(match self.schema.read(cx).dialect {
                                    DialectKind::MySql => "icons/mysql-logo.svg",
                                    DialectKind::PostgreSql => "icons/postgresql-logo.svg",
                                })
                                .size(px(32.0))
                                .text_color(cx.theme().foreground),
                        ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_ellipsis()
                            .font_bold()
                            .child(self.schema.read(cx).name.clone()),
                    ),
            )
            .child(
                div()
                    .p_1()
                    .overflow_y_scrollbar()
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .text_sm()
                            .child("TABLES"),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .text_sm()
                            .child("RELATIONSHIPS"),
                    ),
            )
    }

    fn er_canvas(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
    }

    fn detail_panel(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
    }
}

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_resizable("editor-h-resizable")
            .on_resize(|state, _, cx| {
                let v = state.read(cx);
                cx.global_mut::<AppSettings>().editor_h_pos = Some(v.sizes()[0].as_f32());
                cx.global::<AppSettings>().save();
            })
            .with_handle_appearance(Rc::new(|_, _, _| Some(div().into_any_element())))
            .child(
                resizable_panel()
                    .size(px(cx.global::<AppSettings>().editor_h_pos.unwrap_or(200.0)))
                    .flex_none()
                    .p_1()
                    .child(self.object_list(window, cx)),
            )
            .child(
                resizable_panel().child(
                    v_resizable("editor-v-resizable")
                        .on_resize(|state, _, cx| {
                            let v = state.read(cx);
                            cx.global_mut::<AppSettings>().editor_v_pos = Some(v.sizes()[1].as_f32());
                            cx.global::<AppSettings>().save();
                        })
                        .with_handle_appearance(Rc::new(|_, _, _| Some(div().into_any_element())))
                        .child(resizable_panel().p_1().child(self.er_canvas(window, cx)))
                        .child(
                            resizable_panel()
                                .size(px(cx.global::<AppSettings>().editor_v_pos.unwrap_or(200.0)))
                                .flex_none()
                                .p_1()
                                .child(self.detail_panel(window, cx)),
                        ),
                ),
            )
    }
}
