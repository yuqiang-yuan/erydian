use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dialect::DialectKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttrKind {
    Text { default: Option<String> },
    Select { options: Vec<String> },
    Bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttrValue {
    Text(Option<String>),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub tables: Vec<TableSpec>,
}

impl SchemaDocument {
    pub fn new(dialect: DialectKind, name: impl Into<String>, schema_attrs: BTreeMap<String, AttrValue>) -> Self {
        Self {
            dialect,
            name: name.into(),
            schema_attrs,
            tables: vec![]
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSpec {
    pub id: String,
    pub name: String,
    pub attrs: BTreeMap<String, AttrValue>,
    pub columns: Vec<ColumnSpec>,
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
