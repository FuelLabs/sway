use sway_core::{Engines, TypeId};
use sway_types::Spanned;
use tower_lsp::lsp_types::{Range, Url};

use crate::capabilities::code_actions::{CodeAction, CodeActionContext, CODE_ACTION_DOC_TITLE};

pub(crate) trait GenerateDocCodeAction<'a, T: Spanned>: CodeAction<'a, T> {
    /// Returns a placeholder description as a vector of strings.
    fn description_section(&self) -> Vec<String> {
        vec!["Add a brief description.".to_string()]
    }

    /// Returns a placeholder information section as a vector of strings.
    fn info_section(&self) -> Vec<String> {
        vec![
            String::new(),
            "### Additional Information".to_string(),
            String::new(),
            "Provide information beyond the core purpose or functionality.".to_string(),
        ]
    }

    fn default_template(&self) -> String {
        let lines: Vec<String> = vec![self.description_section(), self.info_section()]
            .into_iter()
            .flatten()
            .collect();
        self.format_lines(lines)
    }

    /// Formats a vector of lines into a doc comment [String].
    fn format_lines(&self, lines: Vec<String>) -> String {
        lines
            .iter()
            .map(|line| format!("{}/// {}\n", self.indentation(), line))
            .collect()
    }

    /// Formats a list item with a name and type into a [String].
    fn formatted_list_item(
        &self,
        engines: &'a Engines,
        name: Option<String>,
        type_id: TypeId,
    ) -> String {
        let name_string = match name {
            Some(name) => format!("`{}`: ", name),
            None => String::new(),
        };
        let type_string = match engines.te().get(type_id).is_unit() {
            true => "()".to_string(),
            false => format!("[{}]", engines.help_out(type_id)),
        };
        format!("* {name_string}{type_string} - Add description here",)
    }
}

pub struct BasicDocCommentCodeAction<'a, T: Spanned> {
    decl: &'a T,
    uri: &'a Url,
}

impl<'a, T: Spanned> GenerateDocCodeAction<'a, T> for BasicDocCommentCodeAction<'a, T> {}

impl<'a, T: Spanned> CodeAction<'a, T> for BasicDocCommentCodeAction<'a, T> {
    fn new(ctx: CodeActionContext<'a>, decl: &'a T) -> Self {
        Self { decl, uri: ctx.uri }
    }

    fn new_text(&self) -> String {
        self.default_template()
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
