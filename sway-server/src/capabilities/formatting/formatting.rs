use std::sync::Arc;

use crate::core::session::Session;
use core_lang::parse;
use lspower::lsp::{DocumentFormattingParams, FormattingOptions, TextDocumentIdentifier, TextEdit};

use super::code_builder::CodeBuilder;

pub fn format_document(
    session: Arc<Session>,
    params: DocumentFormattingParams,
) -> Option<Vec<TextEdit>> {
    let options: FormattingOptions = params.options;
    let text_document: TextDocumentIdentifier = params.text_document;
    let url = text_document.uri;

    session.format_text(&url, options)
}

pub fn get_format_text_edits(text: &str, options: FormattingOptions) -> Option<Vec<TextEdit>> {
    // we only format if code is correct
    match parse(text) {
        core_lang::CompileResult::Ok {
            value: _,
            warnings: _,
            errors: _,
        } => Some(build_edits(text, options)),
        _ => None,
    }
}

fn build_edits(text: &str, options: FormattingOptions) -> Vec<TextEdit> {
    let mut code_builder = CodeBuilder::new(options.tab_size);
    let lines: Vec<&str> = text.split("\n").collect();
    let length_of_lines = lines.len();

    // todo: handle length lines of code
    for line in lines {
        code_builder.format_and_add(line);
    }

    code_builder.to_text_edit(length_of_lines)
}
