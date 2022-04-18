use crate::priv_prelude::*;

#[derive(Debug, Error, Clone, PartialEq, Hash)]
pub enum ParseErrorKind {
    #[error("expected an import name, group of imports, or `*`")]
    ExpectedImportNameGroupOrGlob,
    #[error("expected an item")]
    ExpectedAnItem,
    #[error("expected a comma or closing parenthesis in function arguments")]
    ExpectedCommaOrCloseParenInFnArgs,
    #[error("unrecognized op code")]
    UnrecognizedOpCode,
    #[error("unexpected token in statement")]
    UnexpectedTokenInStatement,
    #[error("this expression cannot be assigned to")]
    UnassignableExpression,
    #[error("unexpected token after array index")]
    UnexpectedTokenAfterArrayIndex,
    #[error("invalid literal to use as a field name")]
    InvalidLiteralFieldName,
    #[error("integer field names cannot have type suffixes")]
    IntFieldWithTypeSuffix,
    #[error("expected a field name")]
    ExpectedFieldName,
    #[error("expected a comma or closing parenthesis in this tuple or parenthesized expression")]
    ExpectedCommaOrCloseParenInTupleOrParenExpression,
    #[error("expected an expression")]
    ExpectedExpression,
    #[error("unexpected token after array length")]
    UnexpectedTokenAfterArrayLength,
    #[error("expected a comma, semicolon or closing bracket when parsing this array")]
    ExpectedCommaSemicolonOrCloseBracketInArray,
    #[error("unexpected token after asm return type")]
    UnexpectedTokenAfterAsmReturnType,
    #[error("malformed asm immediate value")]
    MalformedAsmImmediate,
    #[error("expected an identifier")]
    ExpectedIdent,
    #[error("unexpected token after str length")]
    UnexpectedTokenAfterStrLength,
    #[error("expected a type")]
    ExpectedType,
    #[error("unexpected token after array type length")]
    UnexpectedTokenAfterArrayTypeLength,
    #[error("expected an opening brace")]
    ExpectedOpenBrace,
    #[error("expected an opening parenthesis")]
    ExpectedOpenParen,
    #[error("expected an opening square bracket")]
    ExpectedOpenBracket,
    #[error("expected a literal")]
    ExpectedLiteral,
    #[error("expected a program kind (script, contract, predicate or library)")]
    ExpectedProgramKind,
    #[error("expected `{}`", kinds.iter().map(PunctKind::as_char).collect::<String>())]
    ExpectedPunct { kinds: Vec<PunctKind> },
    #[error("expected `{}`", word)]
    ExpectedKeyword { word: &'static str },
    #[error("unexpected token after abi address")]
    UnexpectedTokenAfterAbiAddress,
}

#[derive(Debug, Error, Clone, PartialEq, Hash)]
#[error("{}", kind)]
pub struct ParseError {
    pub span: Span,
    pub kind: ParseErrorKind,
}
