use gpui_kit::Action;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

gpui_kit::actions!(erydian, [NewSchemaAction]);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Action, JsonSchema)]
pub struct SchemaCreatedAction {
    pub schema_json: String,
}
