//! sway-lsp extensions to the LSP.

use lsp_types::{
    request::Request, TextDocumentContentChangeEvent, TextDocumentIdentifier, Url, WorkspaceEdit,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowAstParams {
    pub text_document: TextDocumentIdentifier,
    pub ast_kind: String,
    pub save_path: Url,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnEnterParams {
    pub text_document: TextDocumentIdentifier,
    /// The actual content changes, including the newline.
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

pub enum OnEnter {}

impl Request for OnEnter {
    type Params = OnEnterParams;
    type Result = Option<WorkspaceEdit>;
    const METHOD: &'static str = "sway/on_enter";
}
