use crate::priv_prelude::*;

pub struct Parser<'a, 'e> {
    token_trees: &'a [TokenTree],
    full_span: Span,
    errors: &'e mut Vec<ParseError>,
}

impl<'a, 'e> Parser<'a, 'e> {
    pub fn new(token_stream: &'a TokenStream, errors: &'e mut Vec<ParseError>) -> Parser<'a, 'e> {
        Parser {
            token_trees: token_stream.token_trees(),
            full_span: token_stream.span(),
            errors,
        }
    }

    pub fn emit_error(&mut self, kind: ParseErrorKind) -> ErrorEmitted {
        let span = match self.token_trees.first() {
            Some(token_tree) => token_tree.span(),
            None => Span::new(
                self.full_span.src().clone(),
                self.full_span.end(),
                self.full_span.end(),
                self.full_span.path().cloned(),
            )
            .unwrap(),
        };
        self.emit_error_with_span(kind, span)
    }

    pub fn emit_error_with_span(&mut self, kind: ParseErrorKind, span: Span) -> ErrorEmitted {
        let error = ParseError { span, kind };
        self.errors.push(error);
        ErrorEmitted { _priv: () }
    }

    pub fn take<P: Peek>(&mut self) -> Option<P> {
        let mut num_tokens = 0;
        let peeker = Peeker {
            token_trees: self.token_trees,
            num_tokens: &mut num_tokens,
        };
        let value = P::peek(peeker)?;
        self.token_trees = &self.token_trees[num_tokens..];
        Some(value)
    }

    pub fn peek<P: Peek>(&self) -> Option<P> {
        let mut num_tokens = 0;
        let peeker = Peeker {
            token_trees: self.token_trees,
            num_tokens: &mut num_tokens,
        };
        let value = P::peek(peeker)?;
        Some(value)
    }

    pub fn peek2<P0: Peek, P1: Peek>(&self) -> Option<(P0, P1)> {
        let mut num_tokens = 0;
        let peeker = Peeker {
            token_trees: self.token_trees,
            num_tokens: &mut num_tokens,
        };
        let value0 = P0::peek(peeker)?;
        let peeker = Peeker {
            token_trees: &self.token_trees[num_tokens..],
            num_tokens: &mut num_tokens,
        };
        let value1 = P1::peek(peeker)?;
        Some((value0, value1))
    }

    pub fn peek3<P0: Peek, P1: Peek, P2: Peek>(&self) -> Option<(P0, P1, P2)> {
        let mut num_tokens_0 = 0;
        let peeker = Peeker {
            token_trees: self.token_trees,
            num_tokens: &mut num_tokens_0,
        };
        let value0 = P0::peek(peeker)?;
        let mut num_tokens_1 = 0;
        let peeker = Peeker {
            token_trees: &self.token_trees[num_tokens_0..],
            num_tokens: &mut num_tokens_1,
        };
        let value1 = P1::peek(peeker)?;
        let mut num_tokens_2 = 0;
        let peeker = Peeker {
            token_trees: &self.token_trees[(num_tokens_0 + num_tokens_1)..],
            num_tokens: &mut num_tokens_2,
        };
        let value2 = P2::peek(peeker)?;
        Some((value0, value1, value2))
    }

    pub fn parse<T: Parse>(&mut self) -> ParseResult<T> {
        T::parse(self)
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
        match self.token_trees.split_first()? {
            (
                TokenTree::Group(Group {
                    delimiter,
                    token_stream,
                    span,
                }),
                rest,
            ) if *delimiter == expected_delimiter => {
                self.token_trees = rest;
                let parser = Parser {
                    token_trees: token_stream.token_trees(),
                    full_span: token_stream.span(),
                    errors: self.errors,
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
        if self.is_empty() {
            Some(ParserConsumed { _priv: PhantomData })
        } else {
            None
        }
    }

    pub fn debug_tokens(&self) -> &[TokenTree] {
        let len = std::cmp::min(5, self.token_trees.len());
        &self.token_trees[..len]
    }
}

pub struct Peeker<'a> {
    token_trees: &'a [TokenTree],
    num_tokens: &'a mut usize,
}

impl<'a> Peeker<'a> {
    pub fn peek_ident(self) -> Result<&'a Ident, Self> {
        match self.token_trees.first() {
            Some(TokenTree::Ident(ident)) => {
                *self.num_tokens = 1;
                Ok(ident)
            }
            _ => Err(self),
        }
    }

    pub fn peek_literal(self) -> Result<&'a Literal, Self> {
        match self.token_trees.first() {
            Some(TokenTree::Literal(literal)) => {
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
        let (last_punct_kind, first_punct_kinds) = match punct_kinds.split_last() {
            Some((last_punct_kind, first_punct_kinds)) => (last_punct_kind, first_punct_kinds),
            None => panic!("peek_punct_kinds called with empty slice"),
        };
        if self.token_trees.len() < punct_kinds.len() {
            return Err(self);
        }
        for (i, punct_kind) in first_punct_kinds.iter().enumerate() {
            match &self.token_trees[i] {
                TokenTree::Punct(Punct {
                    kind,
                    spacing: Spacing::Joint,
                    ..
                }) => {
                    if *kind != *punct_kind {
                        return Err(self);
                    }
                }
                _ => return Err(self),
            }
        }
        let span_end = match &self.token_trees[punct_kinds.len() - 1] {
            TokenTree::Punct(Punct {
                kind,
                spacing,
                span,
            }) => {
                if *kind != *last_punct_kind {
                    return Err(self);
                }
                match spacing {
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
                }
            }
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
        match self.token_trees.first() {
            Some(TokenTree::Group(Group { delimiter, .. })) => {
                *self.num_tokens = 1;
                Ok(*delimiter)
            }
            _ => Err(self),
        }
    }
}

#[derive(Debug)]
pub struct ErrorEmitted {
    _priv: (),
}

pub struct ParserConsumed<'a> {
    _priv: PhantomData<fn(&'a ()) -> &'a ()>,
}

pub type ParseResult<T> = Result<T, ErrorEmitted>;
