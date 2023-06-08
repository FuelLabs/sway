use crate::error::ParseFileError;
use std::path::PathBuf;
use std::sync::Arc;
use sway_ast::{attribute::Annotated, token::CommentedTokenStream, Module};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::SourceEngine;

fn with_handler<T>(
    run: impl FnOnce(&Handler) -> Result<T, ErrorEmitted>,
) -> Result<T, ParseFileError> {
    let handler = <_>::default();
    let res = run(&handler);
    let (errors, _warnings) = handler.consume();
    res.ok()
        .filter(|_| errors.is_empty())
        .ok_or(ParseFileError(errors))
}

pub fn parse_file(
    source_engine: &SourceEngine,
    src: Arc<str>,
    path: Option<Arc<PathBuf>>,
) -> Result<Annotated<Module>, ParseFileError> {
    let source_id = path.map(|p| source_engine.get_source_id(p.as_ref()));
    with_handler(|h| sway_parse::parse_file(h, src, source_id))
}

pub fn lex(input: &Arc<str>) -> Result<CommentedTokenStream, ParseFileError> {
    with_handler(|h| sway_parse::lex_commented(h, input, 0, input.len(), &None))
}

pub fn parse_format<P: sway_parse::Parse + crate::Format>(input: &str) -> String {
    let parsed = with_handler(|handler| {
        let token_stream = sway_parse::lex(handler, &input.into(), 0, input.len(), None)?;
        sway_parse::Parser::new(handler, &token_stream).parse::<P>()
    })
    .unwrap();

    let mut buf = <_>::default();
    parsed.format(&mut buf, &mut <_>::default()).unwrap();
    buf
}

/// Partially parses an AST node that implements sway_parse::Parse.
/// This is used to insert comments locally.
pub fn parse_snippet<P: sway_parse::Parse + crate::Format>(
    input: &str,
) -> Result<P, ParseFileError> {
    with_handler(|handler| {
        let token_stream = sway_parse::lex(handler, &input.into(), 0, input.len(), None)?;
        sway_parse::Parser::new(handler, &token_stream).parse::<P>()
    })
}
