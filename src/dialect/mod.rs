use crate::model::{AttrSpec, Column, ColumnTypeSpec};

mod mysql;
mod postgresql;

pub use mysql::MySqlDialect;
pub use postgresql::PostgreSqlDialect;
use serde::{Deserialize, Serialize};

pub trait Dialect {
    /// The database name
    fn name() -> &'static str;

    /// Additional attributes spec
    fn database_attributes() -> Vec<AttrSpec> {
        vec![]
    }

    /// Additional attributes spec
    fn table_attributes() -> Vec<AttrSpec> {
        vec![]
    }

    /// Supported column data type specs
    fn column_types() -> Vec<ColumnTypeSpec> {
        vec![]
    }

    /// Default pk column when creating a new table
    /// `None` means leave to user to add pk column
    fn default_pk_column() -> Option<Column> {
        None
    }
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

    pub fn table_attributes(&self) -> Vec<AttrSpec> {
        match self {
            Self::MySql => MySqlDialect::table_attributes(),
            Self::PostgreSql => PostgreSqlDialect::table_attributes(),
        }
    }

    pub fn column_types(&self) -> Vec<ColumnTypeSpec> {
        match self {
            Self::MySql => MySqlDialect::column_types(),
            Self::PostgreSql => PostgreSqlDialect::column_types(),
        }
    }

    pub fn default_pk_column(&self) -> Option<Column> {
        match self {
            Self::MySql => MySqlDialect::default_pk_column(),
            Self::PostgreSql => PostgreSqlDialect::default_pk_column(),
        }
    }
}
