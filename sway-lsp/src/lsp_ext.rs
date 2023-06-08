//! sway-lsp extensions to the LSP.

use lsp_types::{request::Request, TextDocumentIdentifier, Url};
use serde::{Deserialize, Serialize};

pub enum ShowAst {}

impl Request for ShowAst {
    type Params = ShowAstParams;
    type Result = String;
    const METHOD: &'static str = "sway/show_ast";
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShowAstParams {
    pub text_document: TextDocumentIdentifier,
    pub ast_kind: String,
    pub save_path: Url,
}
