pub mod abi_decl;
pub mod common;
pub mod constant_decl;
pub mod diagnostic;
pub mod enum_decl;
pub mod enum_variant;
pub mod function_decl;
pub mod storage_field;
pub mod struct_decl;
pub mod struct_field;
pub mod trait_fn;

use crate::{core::{
    token::{Token, TypedAstToken},
    token_map::TokenMap,
}, server_state::CompiledPrograms};
pub use crate::error::DocumentError;
use lsp_types::{
    CodeAction as LspCodeAction, CodeActionDisabled, CodeActionKind, CodeActionOrCommand,
    CodeActionResponse, Diagnostic, Position, Range, TextEdit, Url, WorkspaceEdit,
};
use serde_json::Value;
use std::collections::HashMap;
use sway_core::{language::ty, Engines, Namespace};
use sway_types::{LineCol, Spanned};

pub(crate) const CODE_ACTION_IMPL_TITLE: &str = "Generate impl for";
pub(crate) const CODE_ACTION_NEW_TITLE: &str = "Generate `new`";
pub(crate) const CODE_ACTION_DOC_TITLE: &str = "Generate a documentation template";
pub(crate) const CODE_ACTION_IMPORT_TITLE: &str = "Import";
pub(crate) const CODE_ACTION_QUALIFY_TITLE: &str = "Qualify as";

#[derive(Clone)]
pub(crate) struct CodeActionContext<'a> {
    engines: &'a Engines,
    tokens: &'a TokenMap,
    token: &'a Token,
    uri: &'a Url,
    temp_uri: &'a Url,
    diagnostics: &'a Vec<Diagnostic>,
    namespace: &'a Namespace,
}

pub fn code_actions(
    engines: &Engines,
    token_map: &TokenMap,
    range: &Range,
    uri: &Url,
    temp_uri: &Url,
    diagnostics: &Vec<Diagnostic>,
    compiled_programs: &CompiledPrograms,
) -> Option<CodeActionResponse> {
    let t = token_map.token_at_position(temp_uri, range.start)?;
    let token = t.value();
    let program = compiled_programs.program_from_uri(temp_uri, engines)?;
    let namespace = &program.value().typed.as_ref().unwrap().namespace;

    let ctx = CodeActionContext {
        engines,
        tokens: token_map,
        token,
        uri,
        temp_uri,
        diagnostics,
        namespace,
    };

    let actions_by_type = token
        .as_typed()
        .as_ref()
        .map(|typed_token| match typed_token {
            TypedAstToken::TypedDeclaration(decl) => match decl {
                ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. }) => {
                    abi_decl::code_actions(decl_id, &ctx)
                }
                ty::TyDecl::StructDecl(ty::StructDecl { decl_id, .. }) => {
                    struct_decl::code_actions(decl_id, &ctx)
                }
                ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) => {
                    enum_decl::code_actions(decl_id, &ctx)
                }
                _ => Vec::new(),
            },
            TypedAstToken::TypedFunctionDeclaration(decl) => {
                function_decl::code_actions(decl, &ctx)
            }
            TypedAstToken::TypedStorageField(decl) => storage_field::code_actions(decl, &ctx),
            TypedAstToken::TypedConstantDeclaration(decl) => {
                constant_decl::code_actions(decl, &ctx)
            }
            TypedAstToken::TypedEnumVariant(decl) => enum_variant::code_actions(decl, &ctx),
            TypedAstToken::TypedStructField(decl) => struct_field::code_actions(decl, &ctx),
            TypedAstToken::TypedTraitFn(decl) => trait_fn::code_actions(decl, &ctx),
            _ => Vec::new(),
        })
        .unwrap_or_default();

    let actions_by_diagnostic = diagnostic::code_actions(&ctx).unwrap_or_default();
    Some([actions_by_type, actions_by_diagnostic].concat())
}

pub(crate) trait CodeAction<'a, T: Spanned> {
    /// Creates a new [`CodeAction`] with the given [Engines], declaration type, and [Url].
    fn new(ctx: &CodeActionContext<'a>, decl: &'a T) -> Self;

    /// Returns a [String] of text to insert into the document.
    fn new_text(&self) -> String;

    /// Returns a [String] of text to use as the title of the code action.
    fn title(&self) -> String;

    fn indentation(&self) -> String {
        let LineCol { col, .. } = self.decl().span().start_line_col_one_index();
        " ".repeat(col - 1)
    }

    /// Returns the declaration.
    fn decl(&self) -> &T;

    /// Returns the declaration's [Url].
    fn uri(&self) -> &Url;

    /// Returns an optional [`CodeActionDisabled`] indicating whether this code action should be disabled.
    fn disabled(&self) -> Option<CodeActionDisabled> {
        None
    }

    /// Returns a [`CodeActionOrCommand`] for the given code action.
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
        let LineCol {
            line: last_line, ..
        } = self.decl().span().end_line_col_one_index();
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
        let LineCol {
            line: first_line, ..
        } = self.decl().span().start_line_col_one_index();
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
