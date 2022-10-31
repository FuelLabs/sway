use crate::{Parse, ParseToEnd, Peek};

use core::marker::PhantomData;
use sway_ast::keywords::Keyword;
use sway_ast::literal::Literal;
use sway_ast::token::{
    Delimiter, DocComment, Group, Punct, PunctKind, Spacing, TokenStream, TokenTree,
};
use sway_ast::PubToken;
use sway_error::error::CompileError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_error::parser_error::{ParseError, ParseErrorKind};
use sway_types::{Ident, Span, Spanned};

pub struct Parser<'a, 'e> {
    token_trees: &'a [TokenTree],
    full_span: Span,
    handler: &'e Handler,
}

impl<'a, 'e> Parser<'a, 'e> {
    pub fn new(handler: &'e Handler, token_stream: &'a TokenStream) -> Parser<'a, 'e> {
        Parser {
            token_trees: token_stream.token_trees(),
            full_span: token_stream.span(),
            handler,
        }
    }

    pub fn emit_error(&mut self, kind: ParseErrorKind) -> ErrorEmitted {
        let span = match self.token_trees {
            [token_tree, ..] => token_tree.span(),
            _ => {
                // Create a new span that points to _just_ after the last parsed item or 1
                // character before that if the last parsed item is the last item in the full span.
                let num_trailing_spaces =
                    self.full_span.as_str().len() - self.full_span.as_str().trim_end().len();
                let trim_offset = if num_trailing_spaces == 0 {
                    1
                } else {
                    num_trailing_spaces
                };
                Span::new(
                    self.full_span.src().clone(),
                    self.full_span.end() - trim_offset,
                    self.full_span.end() - trim_offset + 1,
                    self.full_span.path().cloned(),
                )
            }
            .unwrap(),
        };
        self.emit_error_with_span(kind, span)
    }

    pub fn emit_error_with_span(&mut self, kind: ParseErrorKind, span: Span) -> ErrorEmitted {
        let error = ParseError { span, kind };
        self.handler.emit_err(CompileError::Parse { error })
    }

    /// Eats a `P` in its canonical way by peeking.
    ///
    /// Unlike [`Parser::peek`], this method advances the parser on success, but not on failure.
    pub fn take<P: Peek>(&mut self) -> Option<P> {
        let (value, tokens) = Peeker::with(self.token_trees)?;
        self.token_trees = tokens;
        Some(value)
    }

    /// Tries to peek a `P` in its canonical way.
    ///
    /// Either way, on success or failure, the parser is not advanced.
    pub fn peek<P: Peek>(&self) -> Option<P> {
        Peeker::with(self.token_trees).map(|(v, _)| v)
    }

    /// Parses a `T` in its canonical way.
    pub fn parse<T: Parse>(&mut self) -> ParseResult<T> {
        T::parse(self)
    }

    /// Parses `T` given that the guard `G` was successfully peeked.
    ///
    /// Useful to parse e.g., `$keyword $stuff` as a unit where `$keyword` is your guard.
    pub fn guarded_parse<G: Peek, T: Parse>(&mut self) -> ParseResult<Option<T>> {
        self.peek::<G>().map(|_| self.parse()).transpose()
    }

    pub fn parse_to_end<T: ParseToEnd>(self) -> ParseResult<(T, ParserConsumed<'a>)> {
        T::parse_to_end(self)
    }

    pub fn try_parse_to_end<T: Parse>(mut self) -> ParseResult<Option<(T, ParserConsumed<'a>)>> {
        let value = self.parse()?;
        let consumed = match self.check_empty() {
            Some(consumed) => consumed,
            None => return Ok(None),
        };
        Ok(Some((value, consumed)))
    }

    pub fn enter_delimited(
        &mut self,
        expected_delimiter: Delimiter,
    ) -> Option<(Parser<'_, '_>, Span)> {
        match self.token_trees {
            [TokenTree::Group(Group {
                delimiter,
                token_stream,
                span,
            }), rest @ ..]
                if *delimiter == expected_delimiter =>
            {
                self.token_trees = rest;
                let parser = Parser {
                    token_trees: token_stream.token_trees(),
                    full_span: token_stream.span(),
                    handler: self.handler,
                };
                Some((parser, span.clone()))
            }
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.token_trees.is_empty()
    }

    pub fn check_empty(&self) -> Option<ParserConsumed<'a>> {
        self.is_empty()
            .then_some(ParserConsumed { _priv: PhantomData })
    }

    pub fn debug_tokens(&self) -> &[TokenTree] {
        let len = std::cmp::min(5, self.token_trees.len());
        &self.token_trees[..len]
    }

    pub(crate) fn token_trees(&self) -> &'a [TokenTree] {
        self.token_trees
    }

    /// Errors given `Some(PubToken)`.
    pub fn ban_visibility_qualifier(&mut self, vis: &Option<PubToken>) -> ParseResult<()> {
        if let Some(token) = vis {
            return Err(self.emit_error_with_span(
                ParseErrorKind::UnnecessaryVisibilityQualifier {
                    visibility: token.ident(),
                },
                token.span(),
            ));
        }
        Ok(())
    }
}

pub struct Peeker<'a> {
    pub token_trees: &'a [TokenTree],
    num_tokens: &'a mut usize,
}

impl<'a> Peeker<'a> {
    /// Peek a `P` in `token_trees`, if any, and return the `P` + the remainder of the token trees.
    pub fn with<P: Peek>(token_trees: &'a [TokenTree]) -> Option<(P, &'a [TokenTree])> {
        let mut num_tokens = 0;
        let peeker = Peeker {
            token_trees,
            num_tokens: &mut num_tokens,
        };
        let value = P::peek(peeker)?;
        Some((value, &token_trees[num_tokens..]))
    }

    pub fn peek_ident(self) -> Result<&'a Ident, Self> {
        match self.token_trees {
            [TokenTree::Ident(ident), ..] => {
                *self.num_tokens = 1;
                Ok(ident)
            }
            _ => Err(self),
        }
    }

    pub fn peek_literal(self) -> Result<&'a Literal, Self> {
        match self.token_trees {
            [TokenTree::Literal(literal), ..] => {
                *self.num_tokens = 1;
                Ok(literal)
            }
            _ => Err(self),
        }
    }

    pub fn peek_punct_kinds(
        self,
        punct_kinds: &[PunctKind],
        not_followed_by: &[PunctKind],
    ) -> Result<Span, Self> {
        let (last_punct_kind, first_punct_kinds) = punct_kinds
            .split_last()
            .unwrap_or_else(|| panic!("peek_punct_kinds called with empty slice"));
        if self.token_trees.len() < punct_kinds.len() {
            return Err(self);
        }
        for (punct_kind, tt) in first_punct_kinds.iter().zip(self.token_trees.iter()) {
            match tt {
                TokenTree::Punct(Punct {
                    kind,
                    spacing: Spacing::Joint,
                    ..
                }) if *kind == *punct_kind => {}
                _ => return Err(self),
            }
        }
        let span_end = match &self.token_trees[punct_kinds.len() - 1] {
            TokenTree::Punct(Punct {
                kind,
                spacing,
                span,
            }) if *kind == *last_punct_kind => match spacing {
                Spacing::Alone => span,
                Spacing::Joint => match &self.token_trees.get(punct_kinds.len()) {
                    Some(TokenTree::Punct(Punct { kind, .. })) => {
                        if not_followed_by.contains(kind) {
                            return Err(self);
                        }
                        span
                    }
                    _ => span,
                },
            },
            _ => return Err(self),
        };
        let span_start = match &self.token_trees[0] {
            TokenTree::Punct(Punct { span, .. }) => span,
            _ => unreachable!(),
        };
        let span = Span::join(span_start.clone(), span_end.clone());
        *self.num_tokens = punct_kinds.len();
        Ok(span)
    }

    pub fn peek_delimiter(self) -> Result<Delimiter, Self> {
        match self.token_trees {
            [TokenTree::Group(Group { delimiter, .. }), ..] => {
                *self.num_tokens = 1;
                Ok(*delimiter)
            }
            _ => Err(self),
        }
    }

    pub fn peek_doc_comment(self) -> Result<&'a DocComment, Self> {
        match self.token_trees {
            [TokenTree::DocComment(doc_comment), ..] => {
                *self.num_tokens = 1;
                Ok(doc_comment)
            }
            _ => Err(self),
        }
    }
}

pub struct ParserConsumed<'a> {
    _priv: PhantomData<fn(&'a ()) -> &'a ()>,
}

pub type ParseResult<T> = Result<T, ErrorEmitted>;
