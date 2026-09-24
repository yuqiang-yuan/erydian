use std::{collections::{BTreeMap, HashMap}, fmt::Display};

use gpui_kit::SharedString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dialect::DialectKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrKind {
    Text { default: Option<String>, multiple_line: bool },
    Select { options: Vec<SharedString> },
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
    tables: Vec<Table>,

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

    pub fn tables(&self) -> &[Table] {
        &self.tables
    }

    pub fn tables_mut(&mut self) -> &mut [Table] {
        &mut self.tables
    }

    pub fn add_table(&mut self, table: Table) {
        let i = self.tables.len();
        self.tables_index.insert(table.id.clone(), i);
        self.tables.push(table);
    }

    pub fn remove_table(&mut self, id: &str) -> Option<Table> {
        let i = self.tables_index.remove(id)?;
        let table = self.tables.remove(i);

        self.tables_index = self.tables.iter()
            .enumerate()
            .map(|(i, t)| (t.id.clone(), i))
            .collect();
        Some(table)
    }

    pub fn get_table(&self, id: &str) -> Option<&Table> {
        self.tables_index.get(id).and_then(|&i| self.tables.get(i))
    }

    pub fn get_table_mut(&mut self, id: &str) -> Option<&mut Table> {
        let i = *self.tables_index.get(id)?;
        self.tables.get_mut(i)
    }

    pub fn rebuild_index(&mut self) {
        self.tables_index = self.tables.iter()
            .enumerate()
            .map(|(i, t)| (t.id.clone(), i))
            .collect();

        self.tables.iter_mut()
            .for_each(|t| t.rebuild_index());
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
pub struct Table {
    pub id: String,
    pub name: String,
    pub attrs: BTreeMap<String, AttrValue>,
    pub graph: Option<TableGraph>,

    columns: Vec<Column>,
    primary_keys: Vec<PrimaryKey>,
    indexes: Vec<Index>,
    columns_index: HashMap<String, usize>,
}

impl Table {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            attrs: BTreeMap::new(),
            columns: vec![],
            primary_keys: vec![],
            indexes: vec![],
            columns_index: HashMap::new(),
            graph: None,
        }
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn columns_mut(&mut self) -> &mut [Column] {
        &mut self.columns
    }

    pub fn add_column(&mut self, col: Column) {
        let i = self.columns.len();
        self.columns_index.insert(col.id.clone(), i);
        self.columns.push(col)
    }

    pub fn remove_column(&mut self, id: &String) -> Option<Column> {
        let i = self.columns_index.remove(id)?;
        let col = self.columns.remove(i);

        self.columns_index = self.columns.iter()
            .enumerate()
            .map(|(i, t)| (t.id.clone(), i))
            .collect();
        Some(col)
    }

    pub fn get_column(&self, col_id: &String) -> Option<&Column> {
        self.columns_index.get(col_id).map(|i| &self.columns[*i])
    }

    pub fn get_column_mut(&mut self, col_id: &String) -> Option<&mut Column> {
        self.columns_index.get(col_id).map(|i| &mut self.columns[*i])
    }

    pub fn rebuild_index(&mut self) {
        self.columns_index = self.columns.iter()
            .enumerate()
            .map(|(i, t)| (t.id.clone(), i))
            .collect();
    }
}

/// date type categories, for grouping dropdown select or combo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnTypeCategory {
    Number,
    String,
    DateTime,
    Spatial,
}

impl Display for ColumnTypeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnTypeCategory::Number => write!(f, "Number"),
            ColumnTypeCategory::String => write!(f, "String"),
            ColumnTypeCategory::DateTime => write!(f, "DateTime"),
            ColumnTypeCategory::Spatial => write!(f, "Spatial / GIS"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnTypeParam {
    Length,        // VARCHAR(255), BINARY(16)
    Precision,     // DECIMAL(10, _)
    Scale,         // DECIMAL(_, 2)
    EnumValues,    // ENUM('a','b')
    Unsigned,      // INT UNSIGNED（MySQL）
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnTypeSpec {
    pub name: &'static str,
    pub category: ColumnTypeCategory,
    pub params: &'static [ColumnTypeParam],
    pub supports_auto_increment: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ColumnType {
    pub name: String,
    pub length: Option<u32>,
    pub precision: Option<u32>,
    pub scale: Option<u32>,
    pub unsigned: bool,
    pub values: Option<Vec<String>>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: String,
    pub name: String,
    pub column_type: ColumnType,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub auto_increment: bool,
    pub comment: Option<String>,
    pub attrs: BTreeMap<String, AttrValue>,
}

impl Column {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            column_type: ColumnType::default(),
            nullable: true,
            default_value: None,
            auto_increment: false,
            comment: None,
            attrs: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryKey {

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {

}
