use crate::{dialect::Dialect, model::{AttrKind, AttrSpec}};

pub struct PostgreSqlDialect {

}

impl Dialect for PostgreSqlDialect {
    fn name() -> &'static str {
        "postgresql"
    }

    fn database_attributes() -> Vec<crate::model::AttrSpec> {
        vec![
            AttrSpec {
                key: "owner",
                label: "Owner",
                kind: AttrKind::Text { default: None, multiple_line: false },
                required: false,
            }
        ]
    }

    fn table_attributes() -> Vec<AttrSpec> {
        vec![]
    }

    fn column_types() -> Vec<crate::model::ColumnTypeSpec> {
        vec![]
    }
}
