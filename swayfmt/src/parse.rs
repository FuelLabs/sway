use crate::{error::ParseFileError, Formatter, FormatterError};
use sway_ast::{attribute::Annotated, token::CommentedTokenStream, Module};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_features::ExperimentalFeatures;
use sway_types::span::Source;

pub fn with_handler<T>(
    run: impl FnOnce(&Handler) -> Result<T, ErrorEmitted>,
) -> Result<T, ParseFileError> {
    let handler = <_>::default();
    let res = run(&handler);
    let (errors, _warnings, _infos) = handler.consume();
    res.ok()
        .filter(|_| errors.is_empty())
        .ok_or(ParseFileError(errors))
}

pub fn parse_file(
    src: Source,
    experimental: ExperimentalFeatures,
) -> Result<Annotated<Module>, ParseFileError> {
    with_handler(|h| sway_parse::parse_file(h, src, None, experimental))
}

pub fn lex(src: Source) -> Result<CommentedTokenStream, ParseFileError> {
    let end = src.text.len();
    with_handler(|h| sway_parse::lex_commented(h, src, 0, end, &None))
}

pub fn parse_format<P: sway_parse::Parse + crate::Format>(
    input: &str,
    experimental: ExperimentalFeatures,
) -> Result<String, FormatterError> {
    let parsed = with_handler(|handler| {
        let token_stream = sway_parse::lex(handler, input.into(), 0, input.len(), None)?;
        sway_parse::Parser::new(handler, &token_stream, experimental).parse::<P>()
    })?;

    // Allow test cases that include comments.
    let mut formatter = Formatter::default();
    formatter.with_comments_context(input)?;

    let mut buf = <_>::default();
    parsed.format(&mut buf, &mut formatter)?;
    Ok(buf)
}

/// Partially parses an AST node that implements sway_parse::Parse.
/// This is used to insert comments locally.
pub fn parse_snippet<P: sway_parse::Parse + crate::Format>(
    input: &str,
    experimental: ExperimentalFeatures,
) -> Result<P, ParseFileError> {
    with_handler(|handler| {
        let token_stream = sway_parse::lex(handler, input.into(), 0, input.len(), None)?;
        sway_parse::Parser::new(handler, &token_stream, experimental).parse::<P>()
    })
}
