pub mod abi_decl;
pub mod struct_decl;

use crate::core::{session::Session, token::TypedAstToken};
pub use crate::error::DocumentError;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use sway_core::{language::ty::TyDeclaration, transform::AttributesMap, Engines, TypeParameter};
use sway_types::Spanned;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionResponse, Position, Range,
    TextDocumentIdentifier, TextEdit, Url, WorkspaceEdit,
};

pub(crate) const CODE_ACTION_IMPL_TITLE: &str = "Generate impl for";
pub(crate) const CODE_ACTION_NEW_TITLE: &str = "Generate `new`";
pub(crate) const CONTRACT: &str = "Contract";
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

pub(crate) trait CodeActionTrait<'a, T: Spanned> {
    /// Creates a new [CodeActionTrait] with the given [Engines], delcaration type, and [Url].
    fn new(engines: Engines<'a>, decl: &'a T, uri: &'a Url) -> Self;

    /// Returns a [String] of text to insert into the document.
    fn new_text(&self) -> String;

    /// Returns a [String] of text to use as the title of the code action.
    fn title(&self) -> String;

    /// Returns a [String] hold the name of the declaration.
    fn decl_name(&self) -> String;

    /// Returns the declaration.
    fn decl(&self) -> &T;

    /// Returns the declaration's [Url].
    fn uri(&self) -> &Url;

    /// Returns a [CodeActionOrCommand] for the given code action.
    fn code_action(&self) -> CodeActionOrCommand {
        let text_edit = TextEdit {
            range: self.range(),
            new_text: self.new_text(),
        };
        let changes = HashMap::from([(self.uri().clone(), vec![text_edit])]);

        CodeActionOrCommand::CodeAction(CodeAction {
            title: self.title(),
            kind: Some(CodeActionKind::REFACTOR),
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }),
            data: Some(Value::String(self.uri().to_string())),
            ..Default::default()
        })
    }

    /// Returns the [Range] to insert text after the last line of the span, with an empty line in between.
    /// Can be overridden if the code action calls for it.
    fn range(&self) -> Range {
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

    /// Returns an optional [String] of the type parameters for the given [TypeParameter] vector.
    fn type_param_string(&self, type_params: &Vec<TypeParameter>) -> Option<String> {
        if type_params.is_empty() {
            None
        } else {
            Some(
                type_params
                    .iter()
                    .map(|param| param.name_ident.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }
    }

    /// Returns a [String] of a generated impl with the optional `for <for_name>` signature.
    /// Can be used for both ABI and Struct impls.
    fn impl_string(
        &self,
        type_params: Option<String>,
        body: String,
        for_name: Option<String>,
    ) -> String {
        let for_string = match for_name {
            Some(name) => format!(" for {name}"),
            None => "".to_string(),
        };
        let type_param_string = match type_params {
            Some(params) => format!("<{params}>"),
            None => "".to_string(),
        };
        format!(
            "\nimpl{} {}{}{} {{{}}}\n",
            type_param_string,
            self.decl_name(),
            type_param_string,
            for_string,
            body
        )
    }

    /// Returns a [String] of a generated function signature.
    fn fn_signature_string(
        &self,
        fn_name: String,
        params_string: String,
        attr_map: &AttributesMap,
        return_type_string: String,
        body: Option<String>,
    ) -> String {
        let attribute_string = attr_map
            .iter()
            .map(|(_, attrs)| {
                attrs
                    .iter()
                    .map(|attr| format!("{}{}", TAB, attr.span.as_str()))
                    .collect::<Vec<String>>()
                    .join("\n")
            })
            .collect::<Vec<String>>()
            .join("\n");
        let attribute_prefix = match attribute_string.len() > 1 {
            true => "\n",
            false => "",
        };
        let body_string = match body {
            Some(body) => format!(" {body} "),
            None => String::new(),
        };
        format!(
            "{attribute_prefix}{attribute_string}\n{TAB}fn {fn_name}({params_string}){return_type_string} {{{body_string}}}",
        )
    }
}
