use crate::{Parse, ParseResult, Parser, Peek, Peeker};

use sway_ast::{keywords::*, token::OpeningDelimiter};
use sway_error::parser_error::ParseErrorKind;
use sway_types::Spanned;

fn peek_keyword<T: Keyword>(peeker: Peeker<'_>) -> Option<T> {
    let ident = peeker.peek_ident().ok()?;
    (ident.as_str() == T::AS_STR).then(|| T::new(ident.span()))
}

fn parse_keyword<T: Keyword + Peek>(parser: &mut Parser) -> ParseResult<T> {
    match parser.take() {
        Some(value) => Ok(value),
        None => Err(parser.emit_error(ParseErrorKind::ExpectedKeyword { word: T::AS_STR })),
    }
}

macro_rules! keyword_impls {
    ($($ty:ty),*) => {
        $(
            impl Peek for $ty {
                fn peek(peeker: Peeker<'_>) -> Option<Self> {
                    peek_keyword(peeker)
                }
            }

            impl Parse for $ty {
                fn parse(parser: &mut Parser) -> ParseResult<Self> {
                    parse_keyword(parser)
                }
            }
        )*
    };
}

keyword_impls! {
    ScriptToken,
    ContractToken,
    PredicateToken,
    LibraryToken,
    DepToken,
    PubToken,
    UseToken,
    AsToken,
    StructToken,
    ClassToken,
    EnumToken,
    SelfToken,
    FnToken,
    TraitToken,
    ImplToken,
    ForToken,
    AbiToken,
    ConstToken,
    StorageToken,
    StrToken,
    AsmToken,
    ReturnToken,
    IfToken,
    ElseToken,
    MatchToken,
    MutToken,
    LetToken,
    WhileToken,
    WhereToken,
    RefToken,
    DerefToken,
    TrueToken,
    FalseToken,
    BreakToken,
    ContinueToken,
    ConfigurableToken
}

fn peek_token<T: Token>(peeker: Peeker<'_>) -> Option<T> {
    let span = peeker
        .peek_punct_kinds(T::PUNCT_KINDS, T::NOT_FOLLOWED_BY)
        .ok()?;
    Some(T::new(span))
}

fn parse_token<T: Token + Peek>(parser: &mut Parser) -> ParseResult<T> {
    match parser.take() {
        Some(value) => Ok(value),
        None => {
            let kinds = T::PUNCT_KINDS.to_owned();
            Err(parser.emit_error(ParseErrorKind::ExpectedPunct { kinds }))
        }
    }
}

macro_rules! token_impls {
    ($($ty:ty),*) => {
        $(
            impl Peek for $ty {
                fn peek(peeker: Peeker<'_>) -> Option<Self> {
                    peek_token(peeker)
                }
            }

            impl Parse for $ty {
                fn parse(parser: &mut Parser) -> ParseResult<Self> {
                    parse_token(parser)
                }
            }
        )*
    };
}

token_impls! {
    SemicolonToken,
    ForwardSlashToken,
    DoubleColonToken,
    StarToken,
    DoubleStarToken,
    CommaToken,
    ColonToken,
    RightArrowToken,
    LessThanToken,
    GreaterThanToken,
    EqToken,
    AddEqToken,
    SubEqToken,
    StarEqToken,
    DivEqToken,
    ShlEqToken,
    ShrEqToken,
    FatRightArrowToken,
    DotToken,
    DoubleDotToken,
    BangToken,
    PercentToken,
    AddToken,
    SubToken,
    ShrToken,
    ShlToken,
    AmpersandToken,
    CaretToken,
    PipeToken,
    DoubleEqToken,
    BangEqToken,
    GreaterThanEqToken,
    LessThanEqToken,
    DoubleAmpersandToken,
    DoublePipeToken,
    UnderscoreToken,
    HashToken,
    HashBangToken
}

fn peek_open_delimiter<T: OpenDelimiterToken>(peeker: Peeker<'_>) -> Option<T> {
    let span = peeker.peek_open_delimiter_token(T::DELIMITER_KIND).ok()?;
    Some(T::new(span))
}

fn parse_open_delimiter<T: OpenDelimiterToken + Peek>(parser: &mut Parser) -> ParseResult<T> {
    match parser.take() {
        Some(value) => Ok(value),
        None => {
            let err = match T::DELIMITER_KIND {
                [OpeningDelimiter::Parenthesis] => ParseErrorKind::ExpectedOpenParen,
                [OpeningDelimiter::CurlyBrace] => ParseErrorKind::ExpectedOpenBrace,
                [OpeningDelimiter::SquareBracket] => ParseErrorKind::ExpectedOpenBracket,
                [OpeningDelimiter::AngleBracket] => ParseErrorKind::ExpectedOpenBracket,
            };
            Err(parser.emit_error(err))
        }
    }
}

macro_rules! open_delimiter_impls {
    ($($ty:ty),*) => {
        $(
            impl Peek for $ty {
                fn peek(peeker: Peeker<'_>) -> Option<Self> {
                    peek_open_delimiter(peeker)
                }
            }

            impl Parse for $ty {
                fn parse(parser: &mut Parser) -> ParseResult<Self> {
                    parse_open_delimiter(parser)
                }
            }
        )*
    };
}

open_delimiter_impls!(
    OpenParenthesisToken,
    OpenCurlyBraceToken,
    OpenSquareBracketToken,
    OpenAngleBracketToken
);

// Keep this in sync with the list in `sway-ast/keywords.rs` defined by define_keyword!
pub const RESERVED_KEYWORDS: phf::Set<&'static str> = phf::phf_set! {
    "script",
    "contract",
    "predicate",
    "library",
    "dep",
    "pub",
    "use",
    "as",
    "struct",
    "enum",
    "self",
    "fn",
    "trait",
    "impl",
    "for",
    "abi",
    "const",
    "storage",
    "str",
    "asm",
    "return",
    "if",
    "else",
    "match",
    "mut",
    "let",
    "while",
    "where",
    "ref",
    "deref",
    "true",
    "false",
    "break",
    "continue",
    "configurable",
};
