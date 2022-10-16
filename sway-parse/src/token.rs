use core::mem;
use extension_trait::extension_trait;
use num_bigint::BigUint;
use std::path::PathBuf;
use std::sync::Arc;
use sway_ast::literal::{LitChar, LitInt, LitIntType, LitString, Literal};
use sway_ast::token::{
    Comment, CommentedGroup, CommentedTokenStream, CommentedTokenTree, Delimiter, DocComment,
    DocStyle, Punct, PunctKind, Spacing, TokenStream,
};
use sway_error::error::CompileError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_error::lex_error::{LexError, LexErrorKind};
use sway_types::{Ident, Span};
use unicode_xid::UnicodeXID;

#[extension_trait]
impl CharExt for char {
    /// Converts the character into an opening delimiter, if any.
    fn as_open_delimiter(self) -> Option<Delimiter> {
        match self {
            '(' => Some(Delimiter::Parenthesis),
            '{' => Some(Delimiter::Brace),
            '[' => Some(Delimiter::Bracket),
            _ => None,
        }
    }

    /// Converts the character into a closing delimiter, if any.
    fn as_close_delimiter(self) -> Option<Delimiter> {
        match self {
            ')' => Some(Delimiter::Parenthesis),
            '}' => Some(Delimiter::Brace),
            ']' => Some(Delimiter::Bracket),
            _ => None,
        }
    }

    /// Determines what sort of punctuation this character is, if any.
    fn as_punct_kind(self) -> Option<PunctKind> {
        match self {
            ';' => Some(PunctKind::Semicolon),
            ':' => Some(PunctKind::Colon),
            '/' => Some(PunctKind::ForwardSlash),
            ',' => Some(PunctKind::Comma),
            '*' => Some(PunctKind::Star),
            '+' => Some(PunctKind::Add),
            '-' => Some(PunctKind::Sub),
            '<' => Some(PunctKind::LessThan),
            '>' => Some(PunctKind::GreaterThan),
            '=' => Some(PunctKind::Equals),
            '.' => Some(PunctKind::Dot),
            '!' => Some(PunctKind::Bang),
            '%' => Some(PunctKind::Percent),
            '&' => Some(PunctKind::Ampersand),
            '^' => Some(PunctKind::Caret),
            '|' => Some(PunctKind::Pipe),
            '~' => Some(PunctKind::Tilde),
            '_' => Some(PunctKind::Underscore),
            '#' => Some(PunctKind::Sharp),
            _ => None,
        }
    }
}

struct CharIndicesInner<'a> {
    src: &'a str,
    position: usize,
}

impl Iterator for CharIndicesInner<'_> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<(usize, char)> {
        let mut char_indices = self.src[self.position..].char_indices();
        let (_, c) = char_indices.next()?;
        let ret = (self.position, c);
        match char_indices.next() {
            Some((char_width, _)) => self.position += char_width,
            None => self.position = self.src.len(),
        };
        Some(ret)
    }
}

type CharIndices<'a> = std::iter::Peekable<CharIndicesInner<'a>>;
type Result<T> = core::result::Result<T, ErrorEmitted>;

struct Lexer<'l> {
    handler: &'l Handler,
    src: &'l Arc<str>,
    path: &'l Option<Arc<PathBuf>>,
    stream: &'l mut CharIndices<'l>,
}

pub fn lex(
    handler: &Handler,
    src: &Arc<str>,
    start: usize,
    end: usize,
    path: Option<Arc<PathBuf>>,
) -> Result<TokenStream> {
    lex_commented(handler, src, start, end, &path).map(|stream| stream.strip_comments())
}

pub fn lex_commented(
    handler: &Handler,
    src: &Arc<str>,
    start: usize,
    end: usize,
    path: &Option<Arc<PathBuf>>,
) -> Result<CommentedTokenStream> {
    let stream = &mut CharIndicesInner {
        src: &src[..end],
        position: start,
    }
    .peekable();
    let mut l = Lexer {
        handler,
        src,
        path,
        stream,
    };

    let mut parent_token_trees = Vec::new();
    let mut token_trees = Vec::new();
    while let Some((mut index, mut character)) = l.stream.next() {
        if character.is_whitespace() {
            continue;
        }
        if character == '/' {
            match l.stream.peek() {
                Some((_, '/')) => {
                    token_trees.push(lex_line_comment(&mut l, end, index));
                    continue;
                }
                Some((_, '*')) => {
                    if let Some(token) = lex_multiline_comment(&mut l, index) {
                        token_trees.push(token);
                    }
                    continue;
                }
                Some(_) | None => {}
            }
        }
        if character.is_xid_start() || character == '_' {
            // Raw identifier, e.g., `r#foo`? Then mark as such, stripping the prefix `r#`.
            let is_raw_ident = character == 'r' && matches!(l.stream.peek(), Some((_, '#')));
            if is_raw_ident {
                l.stream.next();
                if let Some((next_index, next_character)) = l.stream.next() {
                    character = next_character;
                    index = next_index;
                }
            }

            // Don't accept just `_` as an identifier.
            let not_is_single_underscore = character != '_'
                || l.stream
                    .peek()
                    .map_or(false, |(_, next)| next.is_xid_continue());
            if not_is_single_underscore {
                // Consume until we hit other than `XID_CONTINUE`.
                while l.stream.next_if(|(_, c)| c.is_xid_continue()).is_some() {}
                let ident = Ident::new_with_raw(span_until(&mut l, index), is_raw_ident);
                token_trees.push(CommentedTokenTree::Tree(ident.into()));
                continue;
            }
        }
        if let Some(delimiter) = character.as_open_delimiter() {
            let token_trees = mem::take(&mut token_trees);
            parent_token_trees.push((token_trees, index, delimiter));
            continue;
        }
        if let Some(close_delimiter) = character.as_close_delimiter() {
            match parent_token_trees.pop() {
                None => {
                    // Recover by ignoring the unexpected closing delim,
                    // giving the parser opportunities to realize the need for an opening delim
                    // in e.g., this example:
                    //
                    // fn foo() // <-- Parser expects grouped tokens in `{ ... }` here.
                    //     let x = 0;
                    // } // <- This recovery.
                    let kind = LexErrorKind::UnexpectedCloseDelimiter {
                        position: index,
                        close_delimiter,
                    };
                    let span = span_one(&l, index, character);
                    error(l.handler, LexError { kind, span });
                }
                Some((parent, open_index, open_delimiter)) => {
                    if open_delimiter != close_delimiter {
                        // Recover on e.g., a `{ )` mismatch by having `)` interpreted as `}`.
                        let kind = LexErrorKind::MismatchedDelimiters {
                            open_position: open_index,
                            close_position: index,
                            open_delimiter,
                            close_delimiter,
                        };
                        let span = span_one(&l, index, character);
                        error(l.handler, LexError { kind, span });
                    }
                    token_trees = lex_close_delimiter(
                        &mut l,
                        index,
                        parent,
                        token_trees,
                        open_index,
                        open_delimiter,
                    );
                }
            }
            continue;
        }
        if let Some(token) = lex_string(&mut l, index, character)? {
            token_trees.push(token);
            continue;
        }
        if let Some(token) = lex_char(&mut l, index, character)? {
            token_trees.push(token);
            continue;
        }
        if let Some(token) = lex_int_lit(&mut l, index, character)? {
            token_trees.push(token);
            continue;
        }
        if let Some(token) = lex_punctuation(&mut l, index, character) {
            token_trees.push(token);
            continue;
        }

        // Recover by simply ignoring the character.
        // NOTE(Centril): I'm not sure how good of an idea this is... time will tell.
        let kind = LexErrorKind::InvalidCharacter {
            position: index,
            character,
        };
        let span = span_one(&l, index, character);
        error(l.handler, LexError { kind, span });
        continue;
    }

    // Recover all unclosed delimiters.
    while let Some((parent, open_index, open_delimiter)) = parent_token_trees.pop() {
        let kind = LexErrorKind::UnclosedDelimiter {
            open_position: open_index,
            open_delimiter,
        };
        let span = span_one(&l, open_index, open_delimiter.as_open_char());
        error(l.handler, LexError { kind, span });

        token_trees = lex_close_delimiter(
            &mut l,
            src.len(),
            parent,
            token_trees,
            open_index,
            open_delimiter,
        );
    }
    Ok(CommentedTokenStream {
        token_trees,
        full_span: span(&l, start, end),
    })
}

fn lex_close_delimiter(
    l: &mut Lexer<'_>,
    index: usize,
    mut parent: Vec<CommentedTokenTree>,
    token_trees: Vec<CommentedTokenTree>,
    open_index: usize,
    delimiter: Delimiter,
) -> Vec<CommentedTokenTree> {
    let start_index = open_index + delimiter.as_open_char().len_utf8();
    let full_span = span(l, start_index, index);
    let group = CommentedGroup {
        token_stream: CommentedTokenStream {
            token_trees,
            full_span,
        },
        delimiter,
        span: span_until(l, open_index),
    };
    parent.push(CommentedTokenTree::Tree(group.into()));
    parent
}

fn lex_line_comment(l: &mut Lexer<'_>, end: usize, index: usize) -> CommentedTokenTree {
    let _ = l.stream.next();

    // Find end; either at EOF or at `\n`.
    let end = l
        .stream
        .find(|(_, character)| *character == '\n')
        .map_or(end, |(end, _)| end);
    let sp = span(l, index, end);

    let doc_style = match (sp.as_str().chars().nth(2), sp.as_str().chars().nth(3)) {
        // `//!` is an inner line doc comment.
        (Some('!'), _) => {
            // TODO: Add support for inner line doc comments.
            // Some(DocStyle::Inner)
            None
        }
        // `////` (more than 3 slashes) is not considered a doc comment.
        (Some('/'), Some('/')) => None,
        // `///` is an outer line doc comment.
        (Some('/'), _) => Some(DocStyle::Outer),
        _ => None,
    };

    if let Some(doc_style) = doc_style {
        let doc_comment = DocComment {
            span: sp,
            doc_style,
            content_span: span(l, index + 3, end),
        };
        CommentedTokenTree::Tree(doc_comment.into())
    } else {
        Comment { span: sp }.into()
    }
}

fn lex_multiline_comment(l: &mut Lexer<'_>, index: usize) -> Option<CommentedTokenTree> {
    // Lexing a multi-line comment.
    let _ = l.stream.next();
    let mut unclosed_indices = vec![index];

    let unclosed_multiline_comment = |l: &Lexer<'_>, unclosed_indices: Vec<_>| {
        let span = span(l, *unclosed_indices.last().unwrap(), l.src.len() - 1);
        let kind = LexErrorKind::UnclosedMultilineComment { unclosed_indices };
        error(l.handler, LexError { kind, span });
        None
    };

    loop {
        match l.stream.next() {
            None => return unclosed_multiline_comment(l, unclosed_indices),
            Some((_, '*')) => match l.stream.next() {
                None => return unclosed_multiline_comment(l, unclosed_indices),
                // Matched `*/`, so we're closing some multi-line comment. It could be nested.
                Some((slash_ix, '/')) => {
                    let start = unclosed_indices.pop().unwrap();
                    if unclosed_indices.is_empty() {
                        // For the purposes of lexing,
                        // nested multi-line comments constitute a single multi-line comment.
                        // We could represent them as several ones, but that's unnecessary.
                        let end = slash_ix + '/'.len_utf8();
                        let span = span(l, start, end);
                        return Some(Comment { span }.into());
                    }
                }
                Some(_) => {}
            },
            // Found nested multi-line comment.
            Some((next_index, '/')) => match l.stream.next() {
                None => return unclosed_multiline_comment(l, unclosed_indices),
                Some((_, '*')) => unclosed_indices.push(next_index),
                Some(_) => {}
            },
            Some(_) => {}
        }
    }
}

fn lex_string(
    l: &mut Lexer<'_>,
    index: usize,
    character: char,
) -> Result<Option<CommentedTokenTree>> {
    if character != '"' {
        return Ok(None);
    }
    let mut parsed = String::new();
    loop {
        let unclosed_string_lit = |l: &Lexer<'_>, end| {
            error(
                l.handler,
                LexError {
                    kind: LexErrorKind::UnclosedStringLiteral { position: index },
                    span: span(l, index, end),
                },
            )
        };
        let (_, next_character) = l
            .stream
            .next()
            .ok_or_else(|| unclosed_string_lit(l, l.src.len() - 1))?;
        parsed.push(match next_character {
            '\\' => parse_escape_code(l)
                .map_err(|e| e.unwrap_or_else(|| unclosed_string_lit(l, l.src.len())))?,
            '"' => break,
            _ => next_character,
        });
    }
    let span = span_until(l, index);
    let literal = Literal::String(LitString { span, parsed });
    Ok(Some(CommentedTokenTree::Tree(literal.into())))
}

fn lex_char(
    l: &mut Lexer<'_>,
    index: usize,
    character: char,
) -> Result<Option<CommentedTokenTree>> {
    let is_quote = |c| c == '\'';
    if !is_quote(character) {
        return Ok(None);
    }

    let unclosed_char_lit = |l: &Lexer<'_>| {
        let err = LexError {
            kind: LexErrorKind::UnclosedCharLiteral { position: index },
            span: span(l, index, l.src.len()),
        };
        error(l.handler, err)
    };
    let next = |l: &mut Lexer<'_>| l.stream.next().ok_or_else(|| unclosed_char_lit(l));
    let escape = |l: &mut Lexer<'_>, next_char| {
        if next_char == '\\' {
            parse_escape_code(l).map_err(|e| e.unwrap_or_else(|| unclosed_char_lit(l)))
        } else {
            Ok(next_char)
        }
    };

    let (_, next_char) = next(l)?;
    let parsed = escape(l, next_char)?;

    // Consume the closing `'`.
    let (next_index, next_char) = next(l)?;
    let sp = span_until(l, index);

    // Not a closing quote? Then this is e.g., 'ab'.
    // Most likely the user meant a string literal, so recover as that instead.
    let literal = if !is_quote(next_char) {
        let mut string = String::new();
        string.push(parsed);
        string.push(escape(l, next_char)?);
        loop {
            let (_, next_char) = next(l)?;
            if is_quote(next_char) {
                break;
            }
            string.push(next_char);
        }

        // Emit the expected closing quote error.
        error(
            l.handler,
            LexError {
                kind: LexErrorKind::ExpectedCloseQuote {
                    position: next_index,
                },
                span: span(l, next_index, next_index + string.len()),
            },
        );

        Literal::String(LitString {
            span: sp,
            parsed: string,
        })
    } else {
        Literal::Char(LitChar { span: sp, parsed })
    };

    Ok(Some(CommentedTokenTree::Tree(literal.into())))
}

fn parse_escape_code(l: &mut Lexer<'_>) -> core::result::Result<char, Option<ErrorEmitted>> {
    let error = |kind, span| Err(Some(error(l.handler, LexError { kind, span })));

    match l.stream.next() {
        None => Err(None),
        Some((_, '"')) => Ok('"'),
        Some((_, '\'')) => Ok('\''),
        Some((_, 'n')) => Ok('\n'),
        Some((_, 'r')) => Ok('\r'),
        Some((_, 't')) => Ok('\t'),
        Some((_, '\\')) => Ok('\\'),
        Some((_, '0')) => Ok('\0'),
        Some((index, 'x')) => {
            let (high, low) = match (l.stream.next(), l.stream.next()) {
                (Some((_, high)), Some((_, low))) => (high, low),
                _ => return Err(None),
            };
            let (high, low) = match (high.to_digit(16), low.to_digit(16)) {
                (Some(high), Some(low)) => (high, low),
                _ => return error(LexErrorKind::InvalidHexEscape, span_until(l, index)),
            };
            let parsed_character = char::from_u32((high << 4) | low).unwrap();
            Ok(parsed_character)
        }
        Some((index, 'u')) => {
            match l.stream.next() {
                None => return Err(None),
                Some((_, '{')) => (),
                Some((_, unexpected_char)) => {
                    let span = span_one(l, index, unexpected_char);
                    let kind = LexErrorKind::UnicodeEscapeMissingBrace { position: index };
                    return error(kind, span);
                }
            }
            let mut digits_start_position_opt = None;
            let mut char_value = BigUint::from(0u32);
            let digits_end_position = loop {
                let (position, digit) = match l.stream.next() {
                    None => return Err(None),
                    Some((position, '}')) => break position,
                    Some((position, digit)) => (position, digit),
                };
                if digits_start_position_opt.is_none() {
                    digits_start_position_opt = Some(position);
                };
                let digit = match digit.to_digit(16) {
                    None => {
                        let span = span_one(l, position, digit);
                        let kind = LexErrorKind::InvalidUnicodeEscapeDigit { position };
                        return error(kind, span);
                    }
                    Some(digit) => digit,
                };
                char_value *= 16u32;
                char_value += digit;
            };
            let digits_start_position = digits_start_position_opt.unwrap_or(digits_end_position);
            let char_value = match u32::try_from(char_value) {
                Err(..) => {
                    let span = span(l, digits_start_position, digits_end_position);
                    let kind = LexErrorKind::UnicodeEscapeOutOfRange { position: index };
                    return error(kind, span);
                }
                Ok(char_value) => char_value,
            };
            let parsed_character = match char::from_u32(char_value) {
                None => {
                    let span_all = span_until(l, index);
                    let kind = LexErrorKind::UnicodeEscapeInvalidCharValue { span: span_all };
                    let span = span(l, digits_start_position, digits_end_position);
                    return error(kind, span);
                }
                Some(parsed_character) => parsed_character,
            };
            Ok(parsed_character)
        }
        Some((index, unexpected_char)) => error(
            LexErrorKind::InvalidEscapeCode { position: index },
            span_one(l, index, unexpected_char),
        ),
    }
}

fn lex_int_lit(
    l: &mut Lexer<'_>,
    index: usize,
    character: char,
) -> Result<Option<CommentedTokenTree>> {
    let digit = match character.to_digit(10) {
        None => return Ok(None),
        Some(d) => d,
    };

    let decimal_int_lit = |l, digit: u32| {
        let mut big_uint = BigUint::from(digit);
        let end_opt = parse_digits(&mut big_uint, l, 10);
        (big_uint, end_opt)
    };
    let (big_uint, end_opt) = if digit == 0 {
        let prefixed_int_lit = |l: &mut Lexer<'_>, radix| {
            let _ = l.stream.next();
            let d = l.stream.next();
            let incomplete_int_lit = |end| {
                let kind = match radix {
                    16 => LexErrorKind::IncompleteHexIntLiteral { position: index },
                    8 => LexErrorKind::IncompleteOctalIntLiteral { position: index },
                    2 => LexErrorKind::IncompleteBinaryIntLiteral { position: index },
                    _ => unreachable!(),
                };
                let span = span(l, index, end);
                error(l.handler, LexError { kind, span })
            };
            let (digit_pos, digit) = d.ok_or_else(|| incomplete_int_lit(l.src.len()))?;
            let radix_digit = digit
                .to_digit(radix)
                .ok_or_else(|| incomplete_int_lit(digit_pos))?;
            let mut big_uint = BigUint::from(radix_digit);
            let end_opt = parse_digits(&mut big_uint, l, radix);
            Ok((big_uint, end_opt))
        };

        match l.stream.peek() {
            Some((_, 'x')) => prefixed_int_lit(l, 16)?,
            Some((_, 'o')) => prefixed_int_lit(l, 8)?,
            Some((_, 'b')) => prefixed_int_lit(l, 2)?,
            Some((_, '_' | '0'..='9')) => decimal_int_lit(l, 0),
            Some(&(next_index, _)) => (BigUint::from(0u32), Some(next_index)),
            None => (BigUint::from(0u32), None),
        }
    } else {
        decimal_int_lit(l, digit)
    };
    let literal = Literal::Int(LitInt {
        span: span(l, index, end_opt.unwrap_or(l.src.len())),
        parsed: big_uint,
        ty_opt: lex_int_ty_opt(l)?,
    });
    Ok(Some(CommentedTokenTree::Tree(literal.into())))
}

fn lex_int_ty_opt(l: &mut Lexer<'_>) -> Result<Option<(LitIntType, Span)>> {
    let (suffix_start_position, c) = match l.stream.next_if(|(_, c)| c.is_xid_continue()) {
        None => return Ok(None),
        Some(x) => x,
    };
    let mut suffix = String::from(c);
    let suffix_end_position = loop {
        match l.stream.peek() {
            Some((_, c)) if c.is_xid_continue() => {
                suffix.push(*c);
                let _ = l.stream.next();
            }
            Some((pos, _)) => break *pos,
            None => break l.src.len(),
        }
    };
    // Parse the suffix to a known one, or if unknown, recover by throwing it away.
    let ty = match parse_int_suffix(&suffix) {
        Some(s) => s,
        None => {
            let span = span(l, suffix_start_position, suffix_end_position);
            let kind = LexErrorKind::InvalidIntSuffix {
                suffix: Ident::new(span.clone()),
            };
            error(l.handler, LexError { kind, span });
            return Ok(None);
        }
    };
    let span = span_until(l, suffix_start_position);
    Ok(Some((ty, span)))
}

/// Interpret the given `suffix` string as a `LitIntType`.
fn parse_int_suffix(suffix: &str) -> Option<LitIntType> {
    Some(match suffix {
        "u8" => LitIntType::U8,
        "u16" => LitIntType::U16,
        "u32" => LitIntType::U32,
        "u64" => LitIntType::U64,
        "i8" => LitIntType::I8,
        "i16" => LitIntType::I16,
        "i32" => LitIntType::I32,
        "i64" => LitIntType::I64,
        _ => return None,
    })
}

fn parse_digits(big_uint: &mut BigUint, l: &mut Lexer<'_>, radix: u32) -> Option<usize> {
    loop {
        match l.stream.peek() {
            None => break None,
            Some((_, '_')) => {
                let _ = l.stream.next();
            }
            Some(&(index, character)) => match character.to_digit(radix) {
                None => break Some(index),
                Some(digit) => {
                    let _ = l.stream.next();
                    *big_uint *= radix;
                    *big_uint += digit;
                }
            },
        };
    }
}

fn lex_punctuation(l: &mut Lexer<'_>, index: usize, character: char) -> Option<CommentedTokenTree> {
    let punct = Punct {
        kind: character.as_punct_kind()?,
        spacing: match l.stream.peek() {
            Some((_, next_character)) if next_character.as_punct_kind().is_some() => Spacing::Joint,
            _ => Spacing::Alone,
        },
        span: span_until(l, index),
    };
    Some(CommentedTokenTree::Tree(punct.into()))
}

fn span_until(l: &mut Lexer<'_>, start: usize) -> Span {
    let end = l.stream.peek().map_or(l.src.len(), |(end, _)| *end);
    span(l, start, end)
}

fn span_one(l: &Lexer<'_>, start: usize, c: char) -> Span {
    span(l, start, start + c.len_utf8())
}

fn span(l: &Lexer<'_>, start: usize, end: usize) -> Span {
    Span::new(l.src.clone(), start, end, l.path.clone()).unwrap()
}

/// Emit a lexer error.
fn error(handler: &Handler, error: LexError) -> ErrorEmitted {
    handler.emit_err(CompileError::Lex { error })
}

#[cfg(test)]
mod tests {
    use super::{lex, lex_commented};
    use crate::priv_prelude::*;
    use assert_matches::assert_matches;
    use std::sync::Arc;
    use sway_ast::{
        literal::{LitChar, Literal},
        token::{Comment, CommentedTokenTree, CommentedTree, DocComment, DocStyle, TokenTree},
    };
    use sway_error::handler::Handler;

    #[test]
    fn lex_commented_token_stream() {
        let input = r#"
        //
        // Single-line comment.
        struct Foo {
            /* multi-
             * line-
             * comment */
            bar: i32,
        }
        "#;
        let start = 0;
        let end = input.len();
        let path = None;
        let handler = Handler::default();
        let stream = lex_commented(&handler, &Arc::from(input), start, end, &path).unwrap();
        assert!(handler.into_errors().is_empty());
        let mut tts = stream.token_trees().iter();
        assert_eq!(tts.next().unwrap().span().as_str(), "//");
        assert_eq!(
            tts.next().unwrap().span().as_str(),
            "// Single-line comment."
        );
        assert_eq!(tts.next().unwrap().span().as_str(), "struct");
        assert_eq!(tts.next().unwrap().span().as_str(), "Foo");
        {
            let group = match tts.next() {
                Some(CommentedTokenTree::Tree(CommentedTree::Group(group))) => group,
                _ => panic!("expected group"),
            };
            let mut tts = group.token_stream.token_trees().iter();
            assert_eq!(
                tts.next().unwrap().span().as_str(),
                "/* multi-\n             * line-\n             * comment */",
            );
            assert_eq!(tts.next().unwrap().span().as_str(), "bar");
            assert_eq!(tts.next().unwrap().span().as_str(), ":");
            assert_eq!(tts.next().unwrap().span().as_str(), "i32");
            assert_eq!(tts.next().unwrap().span().as_str(), ",");
            assert!(tts.next().is_none());
        }
        assert!(tts.next().is_none());
    }

    #[test]
    fn lex_doc_comments() {
        let input = r#"
        //none
        ////none
        //!inner
        ///outer
        /// outer
        "#;
        let start = 0;
        let end = input.len();
        let path = None;
        let handler = Handler::default();
        let stream = lex_commented(&handler, &Arc::from(input), start, end, &path).unwrap();
        assert!(handler.into_errors().is_empty());
        let mut tts = stream.token_trees().iter();
        assert_matches!(
            tts.next(),
            Some(CommentedTokenTree::Comment(Comment {
                span
            })) if span.as_str() ==  "//none"
        );
        assert_matches!(
            tts.next(),
            Some(CommentedTokenTree::Comment(Comment {
                span
            })) if span.as_str() ==  "////none"
        );
        // TODO: Add support for inner line doc comments.
        // assert_matches!(
        //     tts.next(),
        //     Some(CommentedTokenTree::Tree(CommentedTree::DocComment(DocComment {
        //         doc_style: DocStyle::Inner,
        //         span,
        //         content_span,
        //     }))) if span.as_str() ==  "//!inner" && content_span.as_str() == "inner"
        // );
        assert_matches!(
            tts.next(),
            Some(CommentedTokenTree::Comment(Comment {
                span
            })) if span.as_str() ==  "//!inner"
        );
        assert_matches!(
            tts.next(),
            Some(CommentedTokenTree::Tree(CommentedTree::DocComment(DocComment {
                doc_style: DocStyle::Outer,
                span,
                content_span
            }))) if span.as_str() ==  "///outer" && content_span.as_str() == "outer"
        );
        assert_matches!(
            tts.next(),
            Some(CommentedTokenTree::Tree(CommentedTree::DocComment(DocComment {
                doc_style: DocStyle::Outer,
                span,
                content_span
            }))) if span.as_str() ==  "/// outer" && content_span.as_str() == " outer"
        );
        assert_eq!(tts.next(), None);
    }

    #[test]
    fn lex_char_escaped_quote() {
        let input = r#"
        '\''
        "#;
        let handler = Handler::default();
        let stream = lex(&handler, &Arc::from(input), 0, input.len(), None).unwrap();
        assert!(handler.into_errors().is_empty());
        let mut tts = stream.token_trees().iter();
        assert_matches!(
            tts.next(),
            Some(TokenTree::Literal(Literal::Char(LitChar {
                parsed: '\'',
                ..
            })))
        );
        assert_eq!(tts.next(), None);
    }
}
