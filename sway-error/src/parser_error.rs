use sway_types::ast::PunctKind;
use sway_types::{Ident, Span};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum ParseErrorKind {
    #[error("Expected an import name, group of imports, or `*`.")]
    ExpectedImportNameGroupOrGlob,
    #[error("Expected an item.")]
    ExpectedAnItem,
    #[error("Expected {} element.",
        if *is_only_documented {
            "a documented"
        } else {
            "an annotated"
        }
    )]
    ExpectedAnAnnotatedElement {
        /// True if the element is only documented with
        /// doc comments but without any other
        /// inner or outer attributes in the annotations.
        is_only_documented: bool,
    },
    #[error("Expected an inner doc comment (`//!`) to be at the top of the module file.")]
    ExpectedInnerDocCommentAtTheTopOfFile,
    #[error("Expected a comma or closing parenthesis in function arguments.")]
    ExpectedCommaOrCloseParenInFnArgs,
    #[error("Unknown assembly instruction.")]
    UnrecognizedOpCode {
        known_op_codes: &'static [&'static str],
    },
    #[error("Unexpected token in statement.")]
    UnexpectedTokenInStatement,
    #[error("This expression cannot be assigned to.")]
    UnassignableExpression {
        /// The friendly name of the kind of the expression
        /// that makes the overall expression unassignable.
        /// E.g., "function call", or "struct instantiation".
        erroneous_expression_kind: &'static str,
        /// [Span] that points to either the whole left-hand
        /// side of the reassignment, or to a [Span] of an
        /// erroneous nested expression, if only a part of
        /// the assignment target expression is erroneous.
        erroneous_expression_span: Span,
    },
    #[error("Unexpected token after array index.")]
    UnexpectedTokenAfterArrayIndex,
    #[error("Invalid literal to use as a field name.")]
    InvalidLiteralFieldName,
    #[error("Invalid statement.")]
    InvalidStatement,
    #[error("Invalid item.")]
    InvalidItem,
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
    #[error("Expected an pattern.")]
    ExpectedPattern,
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
    #[error("Expected a module kind (script, contract, predicate, or library).")]
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
    #[error("Identifier cannot be a reserved keyword.")]
    ReservedKeywordIdentifier,
    #[error("Unnecessary visibility qualifier, `{}` is implied here.", visibility)]
    UnnecessaryVisibilityQualifier { visibility: Ident },
    #[error("Expected a doc comment.")]
    ExpectedDocComment,
    #[error("Use the `struct` keyword to define records, instead of `class`.")]
    UnexpectedClass,
    #[error("Field projections, e.g., `foo.bar` cannot have type arguments.")]
    FieldProjectionWithGenericArgs,
    #[error("Unexpected token after __ptr type.")]
    UnexpectedTokenAfterPtrType,
    #[error("Unexpected token after __slice type.")]
    UnexpectedTokenAfterSliceType,
    #[error("Expected a path type.")]
    ExpectedPathType,
    #[error("Expected ':'. Enum variants must be in the form `Variant: ()`, `Variant: <type>`, or `Variant: (<type1>, ..., <typeN>)`. E.g., `Foo: (), or `Bar: (bool, u32)`.")]
    MissingColonInEnumTypeField,
    #[error("Expected storage key of type U256.")]
    ExpectedStorageKeyU256,
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
#[error("{}", kind)]
pub struct ParseError {
    pub span: Span,
    pub kind: ParseErrorKind,
}
