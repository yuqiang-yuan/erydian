use gpui_kit::base::input::InputState;
use gpui_kit::component::tab::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::model::SchemaDocument;
use crate::view::{AttrState, SelectedItem};

pub struct TableDetailView {
    schema: Entity<SchemaDocument>,
    selected_item: Entity<Option<SelectedItem>>,
    selected_tab_index: usize,
    _subscriptions: Vec<Subscription>,
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
            println!(
                "I'm detail view and the selected item is: {:?}",
                ent.read(cx)
            );
        });

        Self {
            schema,
            selected_item: si.clone(),
            selected_tab_index: 0,
            _subscriptions: vec![si_sub],
        }
    }
}

impl Render for TableDetailView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(
            TabBar::new("table-detail-tabbar")
                .underline()
                .px_2()
                .selected_index(self.selected_tab_index)
                .on_click(cx.listener(|this, idx, _, cx| {
                    this.selected_tab_index = *idx;
                    cx.notify();
                }))
                .child(Tab::new().label("Table"))
                .child(Tab::new().label("Columns"))
                .child(Tab::new().label("Indexes")),
        )
    }
}

/// Panel to show table's attributes
pub struct TablePanel {
    schema: Entity<SchemaDocument>,
    selected_item: Entity<Option<SelectedItem>>,
    name_state: Entity<InputState>,
    attr_states: Vec<AttrState>,
}

impl TablePanel {
    pub fn new(
        schema: Entity<SchemaDocument>,
        selected_item: Entity<Option<SelectedItem>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {

        let attr_states = schema.read(cx)
            .dialect
            .table_attributes()
            .iter()
            .map(|a| AttrState::from_kind(&a.kind, window, cx))
            .collect::<Vec<_>>();

        Self {
            schema,
            selected_item,
            name_state: cx.new(|cx| InputState::new(window, cx)),
            attr_states,
        }
    }
}
