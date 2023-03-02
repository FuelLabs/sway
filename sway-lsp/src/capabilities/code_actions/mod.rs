pub mod abi_decl;
pub mod struct_decl;

use crate::core::{
    session::Session,
    token::{Token, TypedAstToken},
    token_map::TokenMap,
};
pub use crate::error::DocumentError;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use sway_core::{
    language::ty::TyDeclaration,
    transform::{AttributeKind, AttributesMap},
    Engines, TypeParameter,
};
use sway_types::Spanned;
use tower_lsp::lsp_types::{
    CodeAction as LspCodeAction, CodeActionDisabled, CodeActionKind, CodeActionOrCommand,
    CodeActionResponse, Position, Range, TextDocumentIdentifier, TextEdit, Url, WorkspaceEdit,
};

pub(crate) const CODE_ACTION_IMPL_TITLE: &str = "Generate impl for";
pub(crate) const CODE_ACTION_NEW_TITLE: &str = "Generate `new`";
pub(crate) const CONTRACT: &str = "Contract";
pub(crate) const TAB: &str = "    ";

#[derive(Clone)]
pub(crate) struct CodeActionContext<'a> {
    engines: Engines<'a>,
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
    let (_, token) = session
        .token_map()
        .token_at_position(temp_uri, range.start)?;
    let type_engine = session.type_engine.read();
    let decl_engine = session.decl_engine.read();
    let ctx = CodeActionContext {
        engines: Engines::new(&type_engine, &decl_engine),
        tokens: session.token_map(),
        token: &token.clone(),
        uri: &text_document.uri,
    };
    token.typed.and_then(|typed_token| match typed_token {
        TypedAstToken::TypedDeclaration(decl) => match decl {
            TyDeclaration::AbiDeclaration { decl_id, .. } => abi_decl::code_actions(&decl_id, ctx),
            TyDeclaration::StructDeclaration { decl_id, .. } => {
                struct_decl::code_actions(&decl_id, ctx)
            }
            _ => None,
        },
        _ => None,
    })
}

pub(crate) trait CodeAction<'a, T: Spanned> {
    /// Creates a new [CodeAction] with the given [Engines], delcaration type, and [Url].
    fn new(ctx: CodeActionContext<'a>, decl: &'a T) -> Self;

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

    /// Returns a [String] of a an attribute map, optionally excluding comments.
    fn attribute_string(&self, attr_map: &AttributesMap, include_comments: bool) -> String {
        let attr_string = attr_map
            .iter()
            .map(|(kind, attrs)| {
                attrs
                    .iter()
                    .filter_map(|attr| match kind {
                        AttributeKind::DocComment { .. } => {
                            if include_comments {
                                return Some(format!("{}{}", TAB, attr.span.as_str()));
                            }
                            None
                        }
                        _ => Some(format!("{}{}", TAB, attr.span.as_str())),
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            })
            .collect::<Vec<String>>()
            .join("\n");
        let attribute_padding = match attr_string.len() > 1 {
            true => "\n",
            false => "",
        };
        format!("{attr_string}{attribute_padding}")
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
        let attribute_string = self.attribute_string(attr_map, false);
        let body_string = match body {
            Some(body) => format!(" {body} "),
            None => String::new(),
        };
        format!(
            "{attribute_string}{TAB}fn {fn_name}({params_string}){return_type_string} {{{body_string}}}",
        )
    }
}
