pub mod abi_decl;
pub mod common;
pub mod constant_decl;
pub mod enum_decl;
pub mod enum_variant;
pub mod function_decl;
pub mod storage_field;
pub mod struct_decl;
pub mod struct_field;

use crate::core::{
    session::Session,
    token::{Token, TypedAstToken},
    token_map::TokenMap,
};
pub use crate::error::DocumentError;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use sway_core::{language::ty, Engines};
use sway_types::Spanned;
use tower_lsp::lsp_types::{
    CodeAction as LspCodeAction, CodeActionDisabled, CodeActionKind, CodeActionOrCommand,
    CodeActionResponse, Position, Range, TextDocumentIdentifier, TextEdit, Url, WorkspaceEdit,
};

pub(crate) const CODE_ACTION_IMPL_TITLE: &str = "Generate impl for";
pub(crate) const CODE_ACTION_NEW_TITLE: &str = "Generate `new`";
pub(crate) const CODE_ACTION_DOC_TITLE: &str = "Generate a documentation template";

#[derive(Clone)]
pub(crate) struct CodeActionContext<'a> {
    engines: &'a Engines,
    tokens: &'a TokenMap,
    token: &'a Token,
    uri: &'a Url,
}

pub(crate) fn code_actions(
    session: Arc<Session>,
    range: &Range,
    text_document: TextDocumentIdentifier,
    temp_uri: &Url,
) -> Option<CodeActionResponse> {
    let engines = session.engines.read();
    let (_, token) = session
        .token_map()
        .token_at_position(engines.se(), temp_uri, range.start)?;

    let ctx = CodeActionContext {
        engines: &engines,
        tokens: session.token_map(),
        token: &token,
        uri: &text_document.uri,
    };

    match token.typed.as_ref()? {
        TypedAstToken::TypedDeclaration(decl) => match decl {
            ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. }) => {
                abi_decl::code_actions(decl_id, ctx)
            }
            ty::TyDecl::StructDecl(ty::StructDecl { decl_id, .. }) => {
                struct_decl::code_actions(decl_id, ctx)
            }
            ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) => {
                enum_decl::code_actions(decl_id, ctx)
            }
            _ => None,
        },
        TypedAstToken::TypedFunctionDeclaration(decl) => function_decl::code_actions(decl, ctx),
        TypedAstToken::TypedStorageField(decl) => storage_field::code_actions(decl, ctx),
        TypedAstToken::TypedConstantDeclaration(decl) => constant_decl::code_actions(decl, ctx),
        TypedAstToken::TypedEnumVariant(decl) => enum_variant::code_actions(decl, ctx),
        TypedAstToken::TypedStructField(decl) => struct_field::code_actions(decl, ctx),

        _ => None,
    }
}

pub(crate) trait CodeAction<'a, T: Spanned> {
    /// Creates a new [CodeAction] with the given [Engines], delcaration type, and [Url].
    fn new(ctx: CodeActionContext<'a>, decl: &'a T) -> Self;

    /// Returns a [String] of text to insert into the document.
    fn new_text(&self) -> String;

    /// Returns a [String] of text to use as the title of the code action.
    fn title(&self) -> String;

    fn indentation(&self) -> String {
        let (_, column) = self.decl().span().start_pos().line_col();
        " ".repeat(column - 1)
    }

    /// Returns the declaration.
    fn decl(&self) -> &T;

    /// Returns the declaration's [Url].
    fn uri(&self) -> &Url;

    /// Returns an optional [CodeActionDisabled] indicating whether this code action should be disabled.
    fn disabled(&self) -> Option<CodeActionDisabled> {
        None
    }

    /// Returns a [CodeActionOrCommand] for the given code action.
    fn code_action(&self) -> CodeActionOrCommand {
        let text_edit = TextEdit {
            range: self.range(),
            new_text: self.new_text(),
        };
        let changes = HashMap::from([(self.uri().clone(), vec![text_edit])]);

        CodeActionOrCommand::CodeAction(LspCodeAction {
            title: self.title(),
            kind: Some(CodeActionKind::REFACTOR),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }),
            data: Some(Value::String(self.uri().to_string())),
            disabled: self.disabled(),
            ..Default::default()
        })
    }

    /// Returns the [Range] to insert text. This will usually be implemented as `range_before` or `range_after`.
    fn range(&self) -> Range;

    /// Returns the [Range] to insert text after the last line of the span, with an empty line in between.
    fn range_after(&self) -> Range {
        let (last_line, _) = self.decl().span().end_pos().line_col();
        let insertion_position = Position {
            line: last_line as u32,
            character: 0,
        };
        Range {
            start: insertion_position,
            end: insertion_position,
        }
    }

    /// Returns the [Range] to insert text before the first line of the span, with an empty line in between.
    fn range_before(&self) -> Range {
        let (first_line, _) = self.decl().span().start_pos().line_col();
        let insertion_position = Position {
            line: first_line as u32 - 1,
            character: 0,
        };
        Range {
            start: insertion_position,
            end: insertion_position,
        }
    }
}
