mod schema;

pub use schema::SchemaDocument;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub struct AttrSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: AttrKind,
    pub required: bool,
}
