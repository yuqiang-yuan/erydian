use gpui_kit::base::StyledExt;
use gpui_kit::base::input::{InputEvent, InputState, TextareaState};
use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::select::{SearchableVec, Select, SelectGroup, SelectState};
use gpui_kit::component::select::SelectEvent;
use gpui_kit::component::{ChildElement, Size, Sizable, tab::*};
use gpui_kit::component::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::model::{AttrValue, ColumnTypeSpec, SchemaDocument};
use crate::view::{AttrField, AttrFieldOptions, AttrState, SelectedItem};

const TAB_TABLE: usize = 0;
const TAB_COLUMNS: usize = 1;

const COL_WIDTHS: &[Pixels] = &[
    px(60.0),
    px(150.0),
    px(150.0),
    px(80.0),
    px(80.0),
    px(80.0),
    px(80.0),
    px(80.0),
];

pub struct TableDetailView {
    schema: Entity<SchemaDocument>,
    selected_item: Entity<Option<SelectedItem>>,
    selected_tab_index: usize,
    table_panel: Entity<TablePanel>,
    columns_panel: Entity<ColumnsPanel>,
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
            columns_panel: cx.new(|cx| ColumnsPanel::new(schema.clone(), si.clone(), window, cx)),
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
            .child(
                div()
                    .min_h_0()
                    .flex_grow_1()
                    .when(self.selected_tab_index == TAB_TABLE, |this| {
                        this.child(self.table_panel.clone())
                    })
                    .when(self.selected_tab_index == TAB_COLUMNS, |this| {
                        this.child(self.columns_panel.clone())
                    }),
            )
    }
}

pub struct TableSqlPanel {
    schema: Entity<SchemaDocument>,
    selected_item: Entity<Option<SelectedItem>>,
    sql_state: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
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
                AttrField::Text(entity) => cx.subscribe_in(
                    entity,
                    window,
                    |this, state, event: &InputEvent, window, cx| {
                        match event {
                            InputEvent::PressEnter { .. } | InputEvent::Blur => {
                                let s = state.read(cx).value().trim().to_string(); // 先 clone 成 String
                                this.update_attr_value(
                                    attr_state.attr.key,
                                    AttrValue::Text(Some(s)),
                                    window,
                                    cx,
                                );
                            }
                            _ => {}
                        }
                    },
                ),

                AttrField::MultilineText(entity) => cx.subscribe_in(
                    entity,
                    window,
                    |this, state, event: &InputEvent, window, cx| match event {
                        InputEvent::PressEnter { .. } | InputEvent::Blur => {
                            let s = state.read(cx).value().trim().to_string();
                            this.update_attr_value(
                                attr_state.attr.key,
                                AttrValue::Text(Some(s)),
                                window,
                                cx,
                            );
                        }
                        _ => {}
                    },
                ),

                AttrField::Select(entity) => cx.subscribe_in(
                    entity,
                    window,
                    |this, _, event: &SelectEvent<SearchableVec<SharedString>>, window, cx| {
                        match event {
                            SelectEvent::Confirm(v) => {
                                this.update_attr_value(
                                    attr_state.attr.key,
                                    AttrValue::Text(v.clone().map(|s| s.to_string())),
                                    window,
                                    cx,
                                );
                            }
                        }
                    },
                ),

                AttrField::Bool(entity) => {
                    cx.observe_in(entity, window, |this, ent, window, cx| {
                        this.update_attr_value(
                            attr_state.attr.key,
                            AttrValue::Bool(*ent.read(cx)),
                            window,
                            cx,
                        );
                    })
                }
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

        if let Some((name, attrs)) = &table_meta {
            self.name_state.update(cx, |this, cx| {
                this.set_value(name.clone(), window, cx);
            });

            // extra attributes
            self.attr_states.iter_mut().for_each(|state| {
                if let Some(v) = attrs.get(state.attr.key) {
                    state.field.set_value(v.clone(), window, cx);
                } else {
                    state.field.clear_value(window, cx);
                }
            });
        } else {
            self.name_state.update(cx, |this, cx| {
                this.set_value("".to_string(), window, cx);
            });

            // clear extra attributes
            self.attr_states.iter_mut().for_each(|state| {
                state.field.clear_value(window, cx);
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
        attr_value: AttrValue,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        println!(
            "table attribute {} will be updated to {:?}",
            attr_key, attr_value
        );
        let tid = if let Some(SelectedItem::Table(s)) = self.selected_item.read(cx) {
            Some(s.to_string())
        } else {
            None
        };

        if tid.is_none() {
            return;
        }

        let tid = tid.unwrap();

        self.schema.update(cx, |this, _| {
            if let Some(table) = this.get_table_mut(&tid) {
                table.attrs.insert(attr_key.to_string(), attr_value);
                if let Some(g) = &mut table.graph {
                    g.is_dirty = true;
                }
            }
        });

        cx.notify();
    }
}

impl Render for TablePanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("table-detail")
            .size_full()
            .px_4()
            .overflow_y_scrollbar()
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
                div().children(self.attr_states.iter().enumerate().map(|(i, attr_state)| {
                    div()
                        .mt_4()
                        .h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .w_40()
                                .text_right()
                                .text_sm()
                                .child(attr_state.attr.label.to_string()),
                        )
                        .child(attr_state.field.render_component(
                            Some(AttrFieldOptions::new().id(format!("table-detail-{i}"))),
                            cx,
                        ))
                })),
            )
    }
}

/// 把任意元素适配成 Table 系列能接受的子元素
#[derive(IntoElement)]
pub struct TableChild {
    element: AnyElement,
}

impl TableChild {
    pub fn new(element: impl IntoElement) -> Self {
        Self { element: element.into_any_element() }
    }
}

impl Sizable for TableChild {
    fn with_size(mut self, _: impl Into<Size>) -> Self {
        self
    }
}

impl ChildElement for TableChild {
    fn with_ix(mut self, _: usize) -> Self {
        self
    }
}

impl RenderOnce for TableChild {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.element
    }
}

pub struct ColumnsPanel {
    schema: Entity<SchemaDocument>,
    selected_item: Entity<Option<SelectedItem>>,
    column_rows: Vec<Entity<ColumnRow>>,
}

impl ColumnsPanel {
    pub fn new(
        schema: Entity<SchemaDocument>,
        selected_item: Entity<Option<SelectedItem>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let columns = if let Some(SelectedItem::Table(tid)) = &selected_item.read(cx)
            && let Some(t) = schema.read(cx).get_table(tid)
        {
            t.columns().iter().map(|c| c.clone()).collect::<Vec<_>>()
        } else {
            vec![]
        };

        let column_rows = columns
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                cx.new(|cx| {
                    ColumnRow::new(schema.clone(), selected_item.clone(), i, &c.id, window, cx)
                })
            })
            .collect::<Vec<_>>();

        Self {
            schema,
            selected_item,
            column_rows,
        }
    }
}

impl Render for ColumnsPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Table::new()
            .child(
                TableHeader::new().child(
                    TableRow::new()
                        .child(TableHead::new().min_w(COL_WIDTHS[0]).w(COL_WIDTHS[0]).child("#"))
                        .child(TableHead::new().min_w(COL_WIDTHS[1]).w(COL_WIDTHS[1]).child("Name"))
                        .child(TableHead::new().min_w(COL_WIDTHS[2]).w(COL_WIDTHS[2]).child("Type"))
                        .child(TableHead::new().min_w(COL_WIDTHS[3]).w(COL_WIDTHS[3]).child("Length"))
                        .child(TableHead::new().min_w(COL_WIDTHS[4]).w(COL_WIDTHS[4]).child("Presision"))
                        .child(TableHead::new().min_w(COL_WIDTHS[5]).w(COL_WIDTHS[5]).child("Scale"))
                        .child(TableHead::new().min_w(COL_WIDTHS[6]).w(COL_WIDTHS[6]).child("Not Null"))

                ),
            )
            .child(TableBody::new().children(self.column_rows.iter().map(|c| TableChild::new(c.clone()))))
    }
}

/// UI components for a single column
pub struct ColumnRow {
    schema: Entity<SchemaDocument>,
    selected_item: Entity<Option<SelectedItem>>,
    idx: usize,
    column_id: SharedString,
    name_state: Entity<InputState>,
    type_state: Entity<SelectState<SearchableVec<SelectGroup<ColumnTypeSpec>>>>,
}

impl ColumnRow {
    pub fn new(
        schema: Entity<SchemaDocument>,
        selected_item: Entity<Option<SelectedItem>>,
        idx: usize,
        column_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name_state = cx.new(|cx| InputState::new(window, cx));
        let type_state = cx.new(|cx| {
            let items = schema.read(cx).dialect.grouped_column_type_spec();
            SelectState::new(items, None, window, cx)
        });

        let mut this = Self {
            schema,
            selected_item,
            idx,
            column_id: SharedString::from(column_id),
            name_state,
            type_state,
        };

        this.load_data(window, cx);

        this
    }

    fn load_data(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let col = if let Some(SelectedItem::Table(tid)) = &self.selected_item.read(cx)
            && let Some(t) = self.schema.read(cx).get_table(tid)
            && let Some(c) = t.get_column(&self.column_id.to_string()) {
            Some(c.clone())
        } else {
            None
        };

        self.name_state.update(cx, |state, cx| state.set_value(SharedString::new(col.as_ref().map(|c| c.name.as_str()).unwrap_or("")), window, cx));

    }
}

impl Render for ColumnRow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        TableRow::new()
            .child(TableCell::new().min_w(COL_WIDTHS[0]).w(COL_WIDTHS[0]).child(format!("{}", self.idx + 1)))
            .child(TableCell::new().min_w(COL_WIDTHS[1]).w(COL_WIDTHS[1]).child(Input::new(&self.name_state)))
            .child(TableCell::new().child(Select::new(&self.type_state)))
    }
}
