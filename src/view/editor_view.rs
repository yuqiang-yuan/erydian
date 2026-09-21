use gpui_kit::{AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, base::{StyledExt, h_resizable, resizable_panel}, component::{ActiveTheme, Icon}, div, img, px};

use crate::{dialect::DialectKind, model::SchemaDocument};

pub struct EditorView {
    schema: Entity<SchemaDocument>,
}

impl EditorView {
    pub fn new(schema: SchemaDocument, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            schema: cx.new(|_| schema)
        }
    }

    fn object_list(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
            .v_flex()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .p_1()
                    .w_full()
                    .h_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .p_1()
                            .rounded_sm()
                            .bg(cx.theme().list_active)
                            .child(
                                Icon::default().path(match self.schema.read(cx).dialect {
                                    DialectKind::MySql => "icons/mysql-logo.svg",
                                    DialectKind::PostgreSql => "icons/postgresql-logo.svg",
                                })
                                .size(px(32.0))
                                .text_color(cx.theme().foreground)
                            )
                    )
                    .child(
                        div().child(self.schema.read(cx).name.clone())
                    )
            )
    }
}

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_resizable("editor-h-resizable")
            .on_resize(|state, _, cx| {
                let v = state.read(cx);
                println!("{:?}", v.sizes());
            })
            .child(
                resizable_panel()
                    .size(px(200.0))
                    .p_1()
                    .child(self.object_list(window, cx))
            )
            .child(
                resizable_panel()
                    .child("Draw")
            )
    }
}
