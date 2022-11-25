use crate::error::ParseFileError;
use std::path::PathBuf;
use std::sync::Arc;
use sway_ast::{token::CommentedTokenStream, Module};
use sway_error::handler::{ErrorEmitted, Handler};

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

pub fn parse_file(src: Arc<str>, path: Option<Arc<PathBuf>>) -> Result<Module, ParseFileError> {
    with_handler(|h| sway_parse::parse_file(h, src, path))
}

pub fn lex(input: &Arc<str>) -> Result<CommentedTokenStream, ParseFileError> {
    with_handler(|h| sway_parse::lex_commented(h, input, 0, input.len(), &None))
}

#[cfg(test)]
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
