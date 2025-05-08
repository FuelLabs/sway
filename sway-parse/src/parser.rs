use crate::{Parse, ParseToEnd, Peek};
use core::marker::PhantomData;
use std::cell::RefCell;
use sway_ast::keywords::Keyword;
use sway_ast::literal::Literal;
use sway_ast::token::{
    DocComment, GenericTokenTree, Group, Punct, Spacing, TokenStream, TokenTree,
};
use sway_ast::PubToken;
use sway_error::error::CompileError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_error::parser_error::{ParseError, ParseErrorKind};
use sway_features::ExperimentalFeatures;
use sway_types::{
    ast::{Delimiter, PunctKind},
    Ident, Span, Spanned,
};

pub struct Parser<'a, 'e> {
    token_trees: &'a [TokenTree],
    full_span: Span,
    handler: &'e Handler,
    pub check_double_underscore: bool,
    pub experimental: ExperimentalFeatures,
}

impl<'a, 'e> Parser<'a, 'e> {
    pub fn new(
        handler: &'e Handler,
        token_stream: &'a TokenStream,
        experimental: ExperimentalFeatures,
    ) -> Parser<'a, 'e> {
        Parser {
            token_trees: token_stream.token_trees(),
            full_span: token_stream.span(),
            handler,
            check_double_underscore: true,
            experimental,
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
                    self.full_span.end().saturating_sub(trim_offset),
                    (self.full_span.end() + 1).saturating_sub(trim_offset),
                    self.full_span.source_id().cloned(),
                )
                .unwrap_or(Span::dummy())
            }
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

    /// Tries to peek a `P` as the second token in its canonical way.
    ///
    /// Either way, on success or failure, the parser is not advanced.
    pub fn peek_next<P: Peek>(&self) -> Option<P> {
        Peeker::with(&self.token_trees[1..]).map(|(v, _)| v)
    }

    /// This function will fork the current parse, and call the parsing function.
    /// If it succeeds it will sync the original parser with the forked one;
    ///
    /// If it fails it will return a `Recoverer` together with the `ErrorEmitted`.
    ///
    /// This recoverer can be used to put the forked parsed back in track and then
    /// sync the original parser to allow the parsing to continue.
    pub fn call_parsing_function_with_recovery<
        'original,
        T,
        F: FnOnce(&mut Parser<'a, '_>) -> ParseResult<T>,
    >(
        &'original mut self,
        parsing_function: F,
    ) -> Result<T, ParseRecoveryStrategies<'original, 'a, 'e>> {
        let handler = Handler::default();
        let mut fork = Parser {
            token_trees: self.token_trees,
            full_span: self.full_span.clone(),
            handler: &handler,
            check_double_underscore: self.check_double_underscore,
            experimental: self.experimental,
        };

        match parsing_function(&mut fork) {
            Ok(result) => {
                self.token_trees = fork.token_trees;
                self.handler.append(handler);
                Ok(result)
            }
            Err(error) => {
                let Parser {
                    token_trees,
                    full_span,
                    ..
                } = fork;
                Err(ParseRecoveryStrategies {
                    original: RefCell::new(self),
                    handler,
                    fork_token_trees: token_trees,
                    fork_full_span: full_span,
                    error,
                })
            }
        }
    }

    /// This function will fork the current parse, and try to parse
    /// T using the fork. If it succeeds it will sync the original parser with the forked one;
    ///
    /// If it fails it will return a `Recoverer` together with the `ErrorEmitted`.
    ///
    /// This recoverer can be used to put the forked parsed back in track and then
    /// sync the original parser to allow the parsing to continue.
    pub fn parse_with_recovery<'original, T: Parse>(
        &'original mut self,
    ) -> Result<T, ParseRecoveryStrategies<'original, 'a, 'e>> {
        self.call_parsing_function_with_recovery(|p| p.parse())
    }

    /// This function does three things
    /// 1 - it peeks P;
    /// 2 - it forks the current parser and tries to parse
    /// T using this fork. If it succeeds it syncs the original
    /// parser with the forked one;
    /// 3 - if it fails it will return a `Recoverer` together with the `ErrorEmitted`.
    ///
    /// This recoverer can be used to put the forked parsed back in track and then
    /// sync the original parser to allow the parsing to continue.
    pub fn guarded_parse_with_recovery<'original, P: Peek, T: Parse>(
        &'original mut self,
    ) -> Result<Option<T>, ParseRecoveryStrategies<'original, 'a, 'e>> {
        if self.peek::<P>().is_none() {
            return Ok(None);
        }

        let handler = Handler::default();
        let mut fork = Parser {
            token_trees: self.token_trees,
            full_span: self.full_span.clone(),
            handler: &handler,
            check_double_underscore: self.check_double_underscore,
            experimental: self.experimental,
        };

        match fork.parse() {
            Ok(result) => {
                self.token_trees = fork.token_trees;
                self.handler.append(handler);
                Ok(Some(result))
            }
            Err(error) => {
                let Parser {
                    token_trees,
                    full_span,
                    ..
                } = fork;
                Err(ParseRecoveryStrategies {
                    original: RefCell::new(self),
                    handler,
                    fork_token_trees: token_trees,
                    fork_full_span: full_span,
                    error,
                })
            }
        }
    }

    /// Parses a `T` in its canonical way.
    /// Do not advance the parser on failure
    pub fn try_parse<T: Parse>(&mut self, append_diagnostics: bool) -> ParseResult<T> {
        let handler = Handler::default();
        let mut fork = Parser {
            token_trees: self.token_trees,
            full_span: self.full_span.clone(),
            handler: &handler,
            check_double_underscore: self.check_double_underscore,
            experimental: self.experimental,
        };
        let r = match T::parse(&mut fork) {
            Ok(result) => {
                self.token_trees = fork.token_trees;
                Ok(result)
            }
            Err(err) => Err(err),
        };
        if append_diagnostics {
            self.handler.append(handler);
        }
        r
    }

    /// This method is useful if `T` does not impl `ParseToEnd`
    pub fn try_parse_and_check_empty<T: Parse>(
        mut self,
        append_diagnostics: bool,
    ) -> ParseResult<Option<(T, ParserConsumed<'a>)>> {
        let value = self.try_parse(append_diagnostics)?;
        match self.check_empty() {
            Some(consumed) => Ok(Some((value, consumed))),
            None => Ok(None),
        }
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

    /// Do not advance the parser on failure
    pub fn try_parse_to_end<T: ParseToEnd>(
        &mut self,
        append_diagnostics: bool,
    ) -> ParseResult<(T, ParserConsumed<'a>)> {
        let handler = Handler::default();
        let fork = Parser {
            token_trees: self.token_trees,
            full_span: self.full_span.clone(),
            handler: &handler,
            check_double_underscore: self.check_double_underscore,
            experimental: self.experimental,
        };
        let r = T::parse_to_end(fork);
        if append_diagnostics {
            self.handler.append(handler);
        }
        r
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
                    check_double_underscore: self.check_double_underscore,
                    experimental: self.experimental,
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

    pub fn full_span(&self) -> &Span {
        &self.full_span
    }
    /// Consume tokens while its line equals to `line`.
    ///
    /// # Warning
    ///
    /// To calculate lines the original source code needs to be transversed.
    pub fn consume_while_line_equals(&mut self, line: usize) {
        loop {
            let Some(current_token) = self.token_trees.first() else {
                break;
            };

            let current_span = current_token.span();
            let current_span_line = current_span.start_line_col_one_index().line;

            if current_span_line != line {
                break;
            } else {
                self.token_trees = &self.token_trees[1..];
            }
        }
    }

    pub fn has_errors(&self) -> bool {
        self.handler.has_errors()
    }

    pub fn has_warnings(&self) -> bool {
        self.handler.has_warnings()
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
        let span = Span::join(span_start.clone(), span_end);
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

/// This struct is returned by some parser methods that allow
/// parser recovery.
///
/// It implements some standardized recovery strategies or it allows
/// custom strategies using the `start` method.
pub struct ParseRecoveryStrategies<'original, 'a, 'e> {
    original: RefCell<&'original mut Parser<'a, 'e>>,
    handler: Handler,
    fork_token_trees: &'a [TokenTree],
    fork_full_span: Span,
    error: ErrorEmitted,
}

impl<'a> ParseRecoveryStrategies<'_, 'a, '_> {
    /// This strategy consumes everything at the current line and emits the fallback error
    /// if the forked parser does not contain any error.
    pub fn recover_at_next_line_with_fallback_error(
        &self,
        kind: ParseErrorKind,
    ) -> (Box<[Span]>, ErrorEmitted) {
        let line = if self.fork_token_trees.is_empty() {
            None
        } else {
            self.last_consumed_token()
                .map(|x| x.span())
                .or_else(|| self.fork_token_trees.first().map(|x| x.span()))
                .map(|x| x.start_line_col_one_index().line)
        };

        self.start(|p| {
            if let Some(line) = line {
                p.consume_while_line_equals(line);
            }
            if !p.has_errors() {
                p.emit_error_with_span(kind, self.diff_span(p));
            }
        })
    }

    /// Starts the parser recovery process calling the callback with the forked parser.
    /// All the changes to this forked parser will be imposed into the original parser,
    /// including diagnostics.
    pub fn start<'this>(
        &'this self,
        f: impl FnOnce(&mut Parser<'a, 'this>),
    ) -> (Box<[Span]>, ErrorEmitted) {
        let mut p = {
            let original = self.original.borrow();
            Parser {
                token_trees: self.fork_token_trees,
                full_span: self.fork_full_span.clone(),
                handler: &self.handler,
                check_double_underscore: original.check_double_underscore,
                experimental: original.experimental,
            }
        };
        f(&mut p);
        self.finish(p)
    }

    /// This is the token before the whole tentative parser started.
    pub fn starting_token(&self) -> &GenericTokenTree<TokenStream> {
        let original = self.original.borrow();
        &original.token_trees[0]
    }

    /// This is the last consumed token of the forked parser. This the token
    /// immediately before the forked parser head.
    pub fn last_consumed_token(&self) -> Option<&GenericTokenTree<TokenStream>> {
        let fork_head_span = self.fork_token_trees.first()?.span();

        // find the last token consumed by the fork
        let original = self.original.borrow();
        let fork_pos = original
            .token_trees
            .iter()
            .position(|x| x.span() == fork_head_span)?;

        let before_fork_pos = fork_pos.checked_sub(1)?;
        original.token_trees.get(before_fork_pos)
    }

    /// This return a span encopassing all tokens that were consumed by the `p` since the start
    /// of the tentative parsing
    ///
    /// This is useful to show one single error for all the consumed tokens.
    pub fn diff_span<'this>(&self, p: &Parser<'a, 'this>) -> Span {
        let original = self.original.borrow_mut();

        // collect all tokens trees that were consumed by the fork
        let qty = if let Some(first_fork_tt) = p.token_trees.first() {
            original
                .token_trees
                .iter()
                .position(|tt| tt.span() == first_fork_tt.span())
                .expect("not finding fork head")
        } else {
            original.token_trees.len()
        };

        let garbage: Vec<_> = original
            .token_trees
            .iter()
            .take(qty)
            .map(|x| x.span())
            .collect();

        Span::join_all(garbage)
    }

    fn finish(&self, p: Parser<'a, '_>) -> (Box<[Span]>, ErrorEmitted) {
        let mut original = self.original.borrow_mut();

        // collect all tokens trees that were consumed by the fork
        let qty = if let Some(first_fork_tt) = p.token_trees.first() {
            original
                .token_trees
                .iter()
                .position(|tt| tt.span() == first_fork_tt.span())
                .expect("not finding fork head")
        } else {
            original.token_trees.len()
        };

        let garbage: Vec<_> = original
            .token_trees
            .iter()
            .take(qty)
            .map(|x| x.span())
            .collect();

        original.token_trees = p.token_trees;
        original.handler.append(self.handler.clone());

        (garbage.into_boxed_slice(), self.error)
    }
}

pub struct ParserConsumed<'a> {
    _priv: PhantomData<fn(&'a ()) -> &'a ()>,
}

pub type ParseResult<T> = Result<T, ErrorEmitted>;
