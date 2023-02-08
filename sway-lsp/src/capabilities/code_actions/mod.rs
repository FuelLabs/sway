pub mod abi_decl;
pub mod struct_decl;

use crate::core::{session::Session, token::TypedAstToken};
pub use crate::error::DocumentError;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use sway_core::{language::ty::TyDeclaration, Engines};
use sway_types::Span;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionResponse, Position, Range,
    TextDocumentIdentifier, TextEdit, Url, WorkspaceEdit,
};

pub(crate) const TAB: &str = "    ";

pub(crate) fn code_actions(
    session: Arc<Session>,
    range: &Range,
    text_document: TextDocumentIdentifier,
    temp_uri: &Url,
) -> Option<CodeActionResponse> {
    let (_, token) = session
        .token_map()
        .token_at_position(temp_uri, range.start)?;

    token.typed.and_then(|typed_token| match typed_token {
        TypedAstToken::TypedDeclaration(decl) => match decl {
            TyDeclaration::AbiDeclaration(ref decl_id) => abi_decl::code_actions(
                Engines::new(&session.type_engine.read(), &session.decl_engine.read()),
                decl_id,
                &text_document.uri,
            ),
            TyDeclaration::StructDeclaration(ref decl_id) => struct_decl::code_actions(
                Engines::new(&session.type_engine.read(), &session.decl_engine.read()),
                decl_id,
                &text_document.uri,
            ),
            _ => None,
        },
        _ => None,
    })
}

/// Returns the range to insert text after the last line of the span, with an empty line in between.
pub(crate) fn range_after_last_line(span: &Span) -> Range {
    let (last_line, _) = span.end_pos().line_col();
    let insertion_position = Position {
        line: last_line as u32,
        character: 0,
    };
    Range {
        start: insertion_position,
        end: insertion_position,
    }
}

/// Builds a [CodeActionOrCommand] with the given title and edits.
pub(crate) fn build_code_action(
    title: String,
    changes: HashMap<Url, Vec<TextEdit>>,
    uri: &Url,
) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: title,
        kind: Some(CodeActionKind::REFACTOR),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }),
        data: Some(Value::String(uri.to_string())),
        ..Default::default()
    })
}
