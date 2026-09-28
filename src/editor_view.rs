use gpui_kit::{Context, Entity, IntoElement, Render, Window, div};

use crate::model::SchemaDocument;


pub struct EditorView {

}

impl EditorView {
    pub fn new(schema: Entity<SchemaDocument>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
