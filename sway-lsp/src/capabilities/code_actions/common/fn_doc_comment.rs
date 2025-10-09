use crate::capabilities::code_actions::{CodeAction, CodeActionContext, CODE_ACTION_DOC_TITLE};
use lsp_types::{Range, Url};
use sway_core::{language::ty::FunctionSignature, Engines};
use sway_types::{Named, Spanned};

use super::generate_doc::GenerateDocCodeAction;

pub struct FnDocCommentCodeAction<'a, T: Spanned + Named + FunctionSignature> {
    engines: &'a Engines,
    decl: &'a T,
    uri: &'a Url,
}

impl<'a, T: Spanned + Named + FunctionSignature> GenerateDocCodeAction<'a, T>
    for FnDocCommentCodeAction<'a, T>
{
}

impl<'a, T: Spanned + Named + FunctionSignature> CodeAction<'a, T>
    for FnDocCommentCodeAction<'a, T>
{
    fn new(ctx: &CodeActionContext<'a>, decl: &'a T) -> Self {
        Self {
            engines: ctx.engines,
            decl,
            uri: ctx.uri,
        }
    }

    fn new_text(&self) -> String {
        let lines: Vec<String> = vec![
            self.description_section(),
            self.info_section(),
            self.arguments_section(),
            self.returns_section(),
            self.reverts_section(),
            self.storage_access_section(),
            self.examples_section(),
        ]
        .into_iter()
        .flatten()
        .collect();
        self.format_lines(lines)
    }

    fn range(&self) -> Range {
        self.range_before()
    }

    fn title(&self) -> String {
        CODE_ACTION_DOC_TITLE.to_string()
    }

    fn decl(&self) -> &T {
        self.decl
    }

    fn uri(&self) -> &Url {
        self.uri
    }
}

impl<T: Spanned + Named + FunctionSignature> FnDocCommentCodeAction<'_, T> {
    /// Formats the return value of the function into a vector of strings.
    fn reverts_section(&self) -> Vec<String> {
        vec![
            String::new(),
            "### Reverts".to_string(),
            String::new(),
            "* List any cases where the function will revert".to_string(),
        ]
    }

    /// Formats the return value of the function into a vector of strings.
    fn storage_access_section(&self) -> Vec<String> {
        vec![
            String::new(),
            "### Number of Storage Accesses".to_string(),
            String::new(),
            "* Reads: `0`".to_string(),
            "* Writes: `0`".to_string(),
            "* Clears: `0`".to_string(),
        ]
    }

    /// Formats the arguments of the function into a vector of strings.
    fn arguments_section(&self) -> Vec<String> {
        if self.decl.parameters().is_empty() {
            return vec![];
        }
        let mut lines = vec![String::new(), "### Arguments".to_string(), String::new()];
        self.decl.parameters().iter().for_each(|param| {
            lines.push(self.formatted_list_item(
                self.engines,
                Some(param.name.to_string()),
                param.type_argument.type_id,
            ));
        });
        lines
    }

    /// Formats the return value of the function into a vector of strings.
    fn returns_section(&self) -> Vec<String> {
        if self
            .engines
            .te()
            .get(self.decl.return_type().type_id)
            .is_unit()
        {
            return vec![];
        }
        vec![
            String::new(),
            "### Returns".to_string(),
            String::new(),
            self.formatted_list_item(self.engines, None, self.decl.return_type().type_id),
        ]
    }

    /// Generates examples of function usage and formats it into a vector of strings.
    fn examples_section(&self) -> Vec<String> {
        let example_args = self
            .decl
            .parameters()
            .iter()
            .map(|param| param.name.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let example = format!("let x = {}({});", self.decl.name(), example_args);
        vec![
            String::new(),
            "### Examples".to_string(),
            String::new(),
            "```sway".to_string(),
            example,
            "```".to_string(),
        ]
    }
}
