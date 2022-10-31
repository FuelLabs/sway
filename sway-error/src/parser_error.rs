use sway_ast::token::PunctKind;
use sway_types::{Ident, Span};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum ParseErrorKind {
    #[error("Expected an import name, group of imports, or `*`.")]
    ExpectedImportNameGroupOrGlob,
    #[error("Expected an item.")]
    ExpectedAnItem,
    #[error("Cannot doc comment a dependency.")]
    CannotDocCommentDepToken,
    #[error("Expected a comma or closing parenthesis in function arguments.")]
    ExpectedCommaOrCloseParenInFnArgs,
    #[error("Unrecognized op code.")]
    UnrecognizedOpCode,
    #[error("Unexpected token in statement.")]
    UnexpectedTokenInStatement,
    #[error("This expression cannot be assigned to.")]
    UnassignableExpression,
    #[error("Unexpected token after array index.")]
    UnexpectedTokenAfterArrayIndex,
    #[error("Invalid literal to use as a field name.")]
    InvalidLiteralFieldName,
    #[error("Integer field names cannot have type suffixes.")]
    IntFieldWithTypeSuffix,
    #[error("Expected a field name.")]
    ExpectedFieldName,
    #[error("Expected a comma or closing parenthesis in this tuple or parenthesized expression.")]
    ExpectedCommaOrCloseParenInTupleOrParenExpression,
    #[error("Expected an expression.")]
    ExpectedExpression,
    #[error("Unexpected token after array length.")]
    UnexpectedTokenAfterArrayLength,
    #[error("Expected a comma, semicolon or closing bracket when parsing this array.")]
    ExpectedCommaSemicolonOrCloseBracketInArray,
    #[error("Unexpected token after asm return type.")]
    UnexpectedTokenAfterAsmReturnType,
    #[error("Malformed asm immediate value.")]
    MalformedAsmImmediate,
    #[error("Expected an identifier.")]
    ExpectedIdent,
    #[error("Unexpected token after str length.")]
    UnexpectedTokenAfterStrLength,
    #[error("Expected a type.")]
    ExpectedType,
    #[error("Unexpected token after array type length.")]
    UnexpectedTokenAfterArrayTypeLength,
    #[error("Expected an opening brace.")]
    ExpectedOpenBrace,
    #[error("Expected an opening parenthesis.")]
    ExpectedOpenParen,
    #[error("Expected an opening square bracket.")]
    ExpectedOpenBracket,
    #[error("Expected a literal.")]
    ExpectedLiteral,
    #[error("Expected a module kind (script, contract, predicate or library).")]
    ExpectedModuleKind,
    #[error("Expected `{}`.", kinds.iter().map(PunctKind::as_char).collect::<String>())]
    ExpectedPunct { kinds: Vec<PunctKind> },
    #[error("Expected `{}`.", word)]
    ExpectedKeyword { word: &'static str },
    #[error("Unexpected token after abi address.")]
    UnexpectedTokenAfterAbiAddress,
    #[error("Expected an attribute.")]
    ExpectedAnAttribute,
    #[error("Unexpected token after an attribute.")]
    UnexpectedTokenAfterAttribute,
    #[error("Identifiers cannot begin with a double underscore, as that naming convention is reserved for compiler intrinsics.")]
    InvalidDoubleUnderscore,
    #[error("Unexpected rest token, must be at the end of pattern.")]
    UnexpectedRestPattern,
    #[error("Identifiers cannot be a reserved keyword.")]
    ReservedKeywordIdentifier,
    #[error("Unnecessary visibility qualifier, `{}` is implied here.", visibility)]
    UnnecessaryVisibilityQualifier { visibility: Ident },
    #[error("Expected a doc comment.")]
    ExpectedDocComment,
    #[error("Use the `struct` keyword to define records, instead of `class`.")]
    UnexpectedClass,
    #[error("Field projections, e.g., `foo.bar` cannot have type arguments.")]
    FieldProjectionWithGenericArgs,
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
#[error("{}", kind)]
pub struct ParseError {
    pub span: Span,
    pub kind: ParseErrorKind,
}
