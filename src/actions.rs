use std::path::PathBuf;

use gpui_kit::Action;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

gpui_kit::actions!(erydian, [NewSchemaAction, AboutAction, QuitAction, OpenFileAction, SaveFileAction, NewTableAction, NewRelationshipAction]);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Action, JsonSchema)]
pub struct SchemaCreatedAction {
    pub schema_json: String,
}

/// File selected and going to open
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Action, JsonSchema)]
pub struct FileOpenedAction {
    pub path: PathBuf,
}

/// File selected and going to save
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Action, JsonSchema)]
pub struct FileSavedAction {
    pub path: PathBuf,
}
