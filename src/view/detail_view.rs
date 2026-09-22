use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::model::SchemaDocument;

pub struct TableDetailView {
    schema: Entity<SchemaDocument>,
    selected_id: Entity<Option<String>>,
}

impl TableDetailView {
    pub fn new(
        schema: Entity<SchemaDocument>,
        selected_id: Entity<Option<String>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Self {
        Self {
            schema,
            selected_id,
        }
    }
}

impl Render for TableDetailView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
