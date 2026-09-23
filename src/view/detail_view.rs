use gpui_kit::base::StyledExt;
use gpui_kit::base::input::{InputEvent, InputState};
use gpui_kit::component::input::Input;
use gpui_kit::component::select::{SearchableVec};
use gpui_kit::component::select::SelectEvent;
use gpui_kit::component::tab::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::model::{AttrValue, SchemaDocument};
use crate::view::{AttrState, AttrField, SelectedItem};

const TAB_TABLE: usize = 0;
const TAB_COLUMNS: usize = 1;

pub struct TableDetailView {
    schema: Entity<SchemaDocument>,
    selected_item: Entity<Option<SelectedItem>>,
    selected_tab_index: usize,
    table_panel: Entity<TablePanel>,
    _subscriptions: Vec<Subscription>,
}

impl TableDetailView {
    pub fn new(
        schema: Entity<SchemaDocument>,
        selected_item: Entity<Option<SelectedItem>>,
        window: &mut Window,
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
            schema: schema.clone(),
            selected_item: si.clone(),
            selected_tab_index: TAB_TABLE,
            table_panel: cx.new(|cx| TablePanel::new(schema.clone(), si.clone(), window, cx)),
            _subscriptions: vec![si_sub],
        }
    }
}

impl Render for TableDetailView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .size_full()
            .child(
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
            .child(div().flex_grow_1().child(self.table_panel.clone()))
    }
}

/// Panel to show table's attributes
pub struct TablePanel {
    schema: Entity<SchemaDocument>,
    selected_item: Entity<Option<SelectedItem>>,
    name_state: Entity<InputState>,
    attr_states: Vec<AttrState>,
    _subscriptions: Vec<Subscription>,
}

impl TablePanel {
    pub fn new(
        schema: Entity<SchemaDocument>,
        selected_item: Entity<Option<SelectedItem>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut subscriptions = vec![];

        let name_state = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_submit_on_enter(true, cx);
            state
        });

        let name_sub = cx.subscribe_in(
            &name_state,
            window,
            |this, state, event: &InputEvent, window, cx| {
                match event {
                    InputEvent::PressEnter { .. } | InputEvent::Blur => {
                        let name = state.read(cx).value().trim().to_string(); // 先 clone 成 String
                        this.update_name(&name, window, cx);
                    }
                    _ => {}
                }
            },
        );
        subscriptions.push(name_sub);

        let tid_sub = cx.observe_in(&selected_item, window, |this, _, window, cx| {
            this.load_data(window, cx);
        });
        subscriptions.push(tid_sub);

        let attr_states = schema
            .read(cx)
            .dialect
            .table_attributes()
            .iter()
            .map(|a| AttrState::from_attr_spec(a, window, cx))
            .collect::<Vec<_>>();

        let attr_subs = attr_states.iter().map(|attr_state| {
            match &attr_state.field {
                AttrField::Text(entity) => cx.subscribe_in(entity, window, |this, state, event: &InputEvent, window, cx| {
                    match event {
                        InputEvent::PressEnter { .. } | InputEvent::Blur => {
                            let s = state.read(cx).value().trim().to_string(); // 先 clone 成 String
                            this.update_attr_value(attr_state.attr.key, &AttrValue::Text(Some(s)), window, cx);
                        }
                        _ => {}
                    }
                }),
                AttrField::MultilineText(entity) => cx.subscribe_in(entity, window, |this, state, event: &InputEvent, window, cx| {
                    match event {
                        InputEvent::PressEnter { .. } | InputEvent::Blur => {
                            let s = state.read(cx).value().trim().to_string();
                            this.update_attr_value(attr_state.attr.key, &AttrValue::Text(Some(s)), window, cx);
                        }
                        _ => {}
                    }
                }),
                AttrField::Select(entity) => cx.subscribe_in(entity, window, |this, _, event: &SelectEvent<SearchableVec<&'static str>>, window, cx| {
                    match event {
                        SelectEvent::Confirm(v) => {
                            let s = v.map(|s| s.to_string());
                            this.update_attr_value(attr_state.attr.key, &AttrValue::Text(s), window, cx);
                        },
                    }
                }),
                AttrField::Bool(entity) => cx.observe_in(entity, window, |this, ent, window, cx| {
                    this.update_attr_value(attr_state.attr.key, &AttrValue::Bool(*ent.read(cx)), window, cx);
                }),
            }
        });

        subscriptions.extend(attr_subs);

        let mut this = Self {
            schema,
            selected_item,
            name_state,
            attr_states,
            _subscriptions: subscriptions,
        };

        if this.selected_item.read(cx).is_some() {
            this.load_data(window, cx);
        }

        this
    }

    fn load_data(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let table_meta = match &self.selected_item.read(cx) {
            Some(SelectedItem::Table(tid)) => self
                .schema
                .read(cx)
                .get_table(tid)
                .map(|t| (t.name.clone(), t.attrs.clone())),
            _ => None,
        };

        if let Some((name, _)) = &table_meta {
            self.name_state.update(cx, |this, cx| {
                this.set_value(name.clone(), window, cx);
            });
        } else {
            self.name_state.update(cx, |this, cx| {
                this.set_value("".to_string(), window, cx);
            });
        }

        cx.notify();
    }

    fn update_name(&mut self, name: &str, _: &mut Window, cx: &mut Context<Self>) {
        let table_id = if let Some(SelectedItem::Table(tid)) = &self.selected_item.read(cx) {
            Some(tid.clone())
        } else {
            None
        };

        if table_id.is_none() {
            return;
        }

        let table_id = table_id.unwrap();

        self.schema.update(cx, |this, cx| {
            if let Some(table) = this.get_table_mut(&table_id)
                && table.name != name
            {
                table.name = name.to_string();
                if let Some(g) = &mut table.graph {
                    g.is_dirty = true;
                }
            }
            cx.notify();
        });
    }

    fn update_attr_value(
        &mut self,
        attr_key: &str,
        attr_value: &AttrValue,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        println!("table attribute {} will be updated to {:?}", attr_key, attr_value);
    }
}

impl Render for TablePanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_4()
            .child(
                div()
                    .mt_4()
                    .h_flex()
                    .gap_2()
                    .items_center()
                    .child(div().w_40().text_right().text_sm().child("Name"))
                    .child(Input::new(&self.name_state)),
            )
            .child(
                div().children(
                    self.attr_states
                        .iter()
                        .enumerate()
                        .map(|(i, attr_state)| {
                            div()
                                .mt_4()
                                .h_flex()
                                .gap_2()
                                .items_center()
                                .child(div().w_40().text_right().text_sm().child(attr_state.attr.label.to_string()))
                                .child(attr_state.field.render_component(&format!("table-detail-{i}"), cx))
                        }),
                ),
            )
    }
}
