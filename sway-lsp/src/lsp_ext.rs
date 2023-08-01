//! sway-lsp extensions to the LSP.

use lsp_types::{request::Request, TextDocumentContentChangeEvent, TextDocumentIdentifier, Url};
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

pub enum OnEnterRequest {}

impl Request for OnEnterRequest {
    type Params = OnEnterParams;
    type Result = String;
    const METHOD: &'static str = "sway/on_enter";
}
