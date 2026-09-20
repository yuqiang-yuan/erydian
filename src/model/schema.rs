use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{dialect::DialectKind, model::AttrValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
