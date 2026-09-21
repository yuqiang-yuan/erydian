use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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
}

impl SchemaDocument {
    pub fn new(dialect: DialectKind, name: String, schema_attrs: BTreeMap<String, AttrValue>) -> Self {
        Self {
            dialect,
            name,
            schema_attrs,
        }
    }
}
