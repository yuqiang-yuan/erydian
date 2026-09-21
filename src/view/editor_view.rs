use std::rc::Rc;

use crate::{
    dialect::DialectKind, model::SchemaDocument, settings::AppSettings,
    view::diagram_view::DiagramView,
};
use gpui_kit::{
    AppContext, Context, Element, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window,
    WindowBackgroundAppearance::Transparent,
    base::{StyledExt, h_resizable, resizable_panel, v_resizable},
    component::{ActiveTheme, Icon, scroll::ScrollableElement},
    div,
    prelude::FluentBuilder,
    px, relative,
};

pub struct EditorView {
    schema: Entity<SchemaDocument>,
    selected_id: Option<String>,
    diagram_view: Entity<DiagramView>,
}

impl EditorView {
    pub fn new(
        schema: Entity<SchemaDocument>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            schema: schema.clone(),
            selected_id: None,
            diagram_view: cx.new(|cx| DiagramView::new(schema, window, cx)),
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
                            .p_1()
                            .px_2()
                            .child("TABLES"),
                    )
                    .children(self.schema.read(cx).tables.iter().map(|t| {
                        let id_clone = t.id.clone();
                        let is_selected = self.selected_id.as_ref() == Some(&t.id);
                        div()
                            .id(t.id.clone())
                            .when(is_selected, |this| {
                                this.border_color(cx.theme().list_active_border)
                                    .bg(cx.theme().list_active)
                            })
                            .when(!is_selected, |this| {
                                this.hover(|style| {
                                    style
                                        .bg(cx.theme().list_active)
                                        .border_color(cx.theme().transparent)
                                })
                            })
                            .border_1()
                            .rounded_sm()
                            .p_1()
                            .px_2()
                            .line_height(relative(1.3))
                            .child(t.name.clone())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.selected_id = Some(id_clone.clone());
                                cx.notify();
                            }))
                    }))
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .text_sm()
                            .p_1()
                            .px_2()
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
                            cx.global_mut::<AppSettings>().editor_v_pos =
                                Some(v.sizes()[1].as_f32());
                            cx.global::<AppSettings>().save();
                        })
                        .with_handle_appearance(Rc::new(|_, _, _| Some(div().into_any_element())))
                        .child(
                            resizable_panel().p_1().child(
                                div()
                                    .size_full()
                                    .v_flex()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .child(self.diagram_view.clone()),
                            ),
                        )
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
