use crate::{Span, Token};
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, stream::Stream};
use generational_arena::Index;
use std::{collections::HashMap, env, fmt, fs};

pub(crate) struct Lexer {}

impl Lexer {
    pub(crate) fn lex(
        input: &str,
        file_ix: Index,
    ) -> impl Parser<char, Vec<(Token, Span)>, Error = Simple<char>> {
        // A parser for numbers
        let num = text::int(10)
            .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
            .collect::<String>()
            .map(Token::Num);

        // A parser for strings
        let str_ = just('"')
            .ignore_then(filter(|c| *c != '"').repeated())
            .then_ignore(just('"'))
            .collect::<String>()
            .map(Token::Str);

        // A parser for operators
        let op = one_of("+-*/!=".chars())
            .repeated()
            .at_least(1)
            .collect::<String>()
            .map(Token::Op);

        // A parser for control characters (delimiters, semicolons, etc.)
        let ctrl = one_of("()[]{};,".chars()).map(|c| Token::Ctrl(c));

        // A parser for identifiers and keywords
        let ident = text::ident().map(|ident: String| match ident.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "print" => Token::Print,
            "if" => Token::If,
            "else" => Token::Else,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "null" => Token::Null,
            _ => Token::Ident(ident),
        });

        // A single token can be one of the above
        let token = num
            .or(str_)
            .or(op)
            .or(ctrl)
            .or(ident)
            .recover_with(skip_then_retry_until([]));

        token
            .map_with_span(move |tok, span: core::ops::Range<usize>| {
                (tok, Span::new_from_idx(file_ix, span.start, span.end))
            })
            .padded()
            .repeated()
    }
}
