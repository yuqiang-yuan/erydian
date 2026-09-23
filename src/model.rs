use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dialect::DialectKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrKind {
    Text { default: Option<String>, multiple_line: bool },
    Select { options: Vec<String> },
    Bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttrValue {
    Text(Option<String>),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct AttrSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: AttrKind,
    pub required: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaDocument {
    pub dialect: DialectKind,
    pub name: String,
    pub schema_attrs: BTreeMap<String, AttrValue>,
    tables: Vec<TableSpec>,

    #[serde(skip)]
    tables_index: HashMap<String, usize>
}

impl SchemaDocument {
    pub fn new(dialect: DialectKind, name: impl Into<String>, schema_attrs: BTreeMap<String, AttrValue>) -> Self {
        Self {
            dialect,
            name: name.into(),
            schema_attrs,
            tables: vec![],
            tables_index: HashMap::new(),
        }
    }

    pub fn tables(&self) -> &[TableSpec] {
        &self.tables
    }

    pub fn tables_mut(&mut self) -> &mut [TableSpec] {
        &mut self.tables
    }

    pub fn add_table(&mut self, table: TableSpec) {
        let i = self.tables.len();
        self.tables_index.insert(table.id.clone(), i);
        self.tables.push(table);
    }

    pub fn remove_table(&mut self, id: &str) -> Option<TableSpec> {
        let i = self.tables_index.remove(id)?;
        let table = self.tables.remove(i);

        self.tables_index = self.tables.iter()
            .enumerate()
            .map(|(i, t)| (t.id.clone(), i))
            .collect();
        Some(table)
    }

    pub fn get_table(&self, id: &str) -> Option<&TableSpec> {
        self.tables_index.get(id).and_then(|&i| self.tables.get(i))
    }

    pub fn get_table_mut(&mut self, id: &str) -> Option<&mut TableSpec> {
        let i = *self.tables_index.get(id)?;
        self.tables.get_mut(i)
    }

    pub fn rebuild_index(&mut self) {
        self.tables_index = self.tables.iter()
            .enumerate()
            .map(|(i, t)| (t.id.clone(), i))
            .collect();
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height
        }
    }

    pub fn center(&self) -> Point {
        Point {
            x: (self.left + self.width) / 2.0,
            y: (self.top + self.height) / 2.0,
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        self.left <= point.x
        && self.left + self.width >= point.x
        && self.top <= point.y
        && self.top + self.height >= point.y
    }
}

/// data related to the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableGraph {
    pub is_dirty: bool,
    pub selected: bool,
    pub rect: Rect,
}

impl TableGraph {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSpec {
    pub id: String,
    pub name: String,
    pub attrs: BTreeMap<String, AttrValue>,
    pub columns: Vec<ColumnSpec>,
    pub graph: Option<TableGraph>,
}

impl TableSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            attrs: BTreeMap::new(),
            columns: vec![
                ColumnSpec::new("id"),
                ColumnSpec::new("name"),
                ColumnSpec::new("sex"),
                ColumnSpec::new("badge_number"),
            ],
            graph: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSpec {
    pub id: String,
    pub name: String,
    pub attrs: BTreeMap<String, AttrValue>,
}

impl ColumnSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            attrs: BTreeMap::new(),
        }
    }
}
