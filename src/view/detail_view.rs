use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::model::SchemaDocument;
use crate::view::SelectedItem;

pub struct TableDetailView {
    schema: Entity<SchemaDocument>,
    selected_item: Entity<Option<SelectedItem>>,
    _subscriptions: Vec<Subscription>
}

impl TableDetailView {
    pub fn new(
        schema: Entity<SchemaDocument>,
        selected_item: Entity<Option<SelectedItem>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let si = selected_item.clone();
        let si_sub = cx.observe(&si, |_, ent, cx| {
            println!("I'm detail view and the selected item is: {:?}", ent.read(cx));
        });

        Self {
            schema,
            selected_item: si.clone(),
            _subscriptions: vec![si_sub]
        }
    }
}

impl Render for TableDetailView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child("this is the table's detail")
    }
}
