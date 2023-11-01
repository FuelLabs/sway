//! sway-lsp extensions to the LSP.

use lsp_types::{TextDocumentContentChangeEvent, TextDocumentIdentifier, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowAstParams {
    pub text_document: TextDocumentIdentifier,
    pub ast_kind: String,
    pub save_path: Url,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnEnterParams {
    pub text_document: TextDocumentIdentifier,
    /// The actual content changes, including the newline.
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualizeParams {
    pub text_document: TextDocumentIdentifier,
    pub graph_kind: String,
}
