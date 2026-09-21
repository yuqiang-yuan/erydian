use crate::{model::AttrSpec};

mod mysql;
mod postgresql;

pub use mysql::MySqlDialect;
pub use postgresql::PostgreSqlDialect;
use serde::{Deserialize, Serialize};

pub trait Dialect {
    fn name() -> &'static str;
    fn database_attributes() -> Vec<AttrSpec>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialectKind {
    #[default]
    #[serde(rename = "mysql")]
    MySql,

    #[serde(rename = "postgresql")]
    PostgreSql,
}

impl DialectKind {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "mysql" => Some(Self::MySql),
            "postgresql" => Some(Self::PostgreSql),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::MySql => MySqlDialect::name(),
            Self::PostgreSql => PostgreSqlDialect::name(),
        }
    }

    pub fn database_attributes(&self) -> Vec<AttrSpec> {
        match self {
            Self::MySql => MySqlDialect::database_attributes(),
            Self::PostgreSql => PostgreSqlDialect::database_attributes(),
        }
    }
}
