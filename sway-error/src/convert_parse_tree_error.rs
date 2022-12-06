use sway_types::{Ident, Span, Spanned};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConvertParseTreeError {
    #[error("pub use imports are not supported")]
    PubUseNotSupported { span: Span },
    #[error("functions used in applications may not be arbitrary expressions")]
    FunctionArbitraryExpression { span: Span },
    #[error("generics are not supported here")]
    GenericsNotSupportedHere { span: Span },
    #[error("tuple index out of range")]
    TupleIndexOutOfRange { span: Span },
    #[error("shift-left expressions are not implemented")]
    ShlNotImplemented { span: Span },
    #[error("shift-right expressions are not implemented")]
    ShrNotImplemented { span: Span },
    #[error("bitwise xor expressions are not implemented")]
    BitXorNotImplemented { span: Span },
    #[error("integer literals in this position cannot have a type suffix")]
    IntTySuffixNotSupported { span: Span },
    #[error("int literal out of range")]
    IntLiteralOutOfRange { span: Span },
    #[error("expected an integer literal")]
    IntLiteralExpected { span: Span },
    #[error("qualified path roots are not implemented")]
    QualifiedPathRootsNotImplemented { span: Span },
    #[error("char literals are not implemented")]
    CharLiteralsNotImplemented { span: Span },
    #[error("hex literals must have 1..16 or 64 digits")]
    HexLiteralLength { span: Span },
    #[error("binary literals must have either 1..64 or 256 digits")]
    BinaryLiteralLength { span: Span },
    #[error("u8 literal out of range")]
    U8LiteralOutOfRange { span: Span },
    #[error("u16 literal out of range")]
    U16LiteralOutOfRange { span: Span },
    #[error("u32 literal out of range")]
    U32LiteralOutOfRange { span: Span },
    #[error("u64 literal out of range")]
    U64LiteralOutOfRange { span: Span },
    #[error("signed integers are not supported")]
    SignedIntegersNotSupported { span: Span },
    #[error("ref variables are not supported")]
    RefVariablesNotSupported { span: Span },
    #[error("literal patterns not supported in this position")]
    LiteralPatternsNotSupportedHere { span: Span },
    #[error("constant patterns not supported in this position")]
    ConstantPatternsNotSupportedHere { span: Span },
    #[error("constructor patterns not supported in this position")]
    ConstructorPatternsNotSupportedHere { span: Span },
    #[error("struct patterns not supported in this position")]
    StructPatternsNotSupportedHere { span: Span },
    #[error("wildcard patterns not supported in this position")]
    WildcardPatternsNotSupportedHere { span: Span },
    #[error("tuple patterns not supported in this position")]
    TuplePatternsNotSupportedHere { span: Span },
    #[error("ref patterns not supported in this position")]
    RefPatternsNotSupportedHere { span: Span },
    #[error("constructor patterns require a single argument")]
    ConstructorPatternOneArg { span: Span },
    #[error("constructor patterns cannot contain sub-patterns")]
    ConstructorPatternSubPatterns { span: Span },
    #[error("paths are not supported in this position")]
    PathsNotSupportedHere { span: Span },
    #[error("Fully specified types are not supported in this position. Try importing the type and referring to it here.")]
    FullySpecifiedTypesNotSupported { span: Span },
    #[error("ContractCaller requires exactly one generic argument")]
    ContractCallerOneGenericArg { span: Span },
    #[error("ContractCaller requires a named type for its generic argument")]
    ContractCallerNamedTypeGenericArg { span: Span },
    #[error("invalid argument for '{attribute}' attribute")]
    InvalidAttributeArgument { attribute: String, span: Span },
    #[error("cannot find type \"{ty_name}\" in this scope")]
    ConstrainedNonExistentType { ty_name: Ident, span: Span },
    #[error("__get_storage_key does not take arguments")]
    GetStorageKeyTooManyArgs { span: Span },
    #[error("recursive types are not supported")]
    RecursiveType { span: Span },
    #[error("enum variant \"{name}\" already declared")]
    DuplicateEnumVariant { name: Ident, span: Span },
    #[error("storage field \"{name}\" already declared")]
    DuplicateStorageField { name: Ident, span: Span },
    #[error("configurable field \"{name}\" already declared")]
    DuplicateConfigurableField { name: Ident, span: Span },
    #[error("struct field \"{name}\" already declared")]
    DuplicateStructField { name: Ident, span: Span },
    #[error("identifier \"{name}\" bound more than once in this parameter list")]
    DuplicateParameterIdentifier { name: Ident, span: Span },
    #[error("self parameter is not allowed for a free function")]
    SelfParameterNotAllowedForFreeFn { span: Span },
    #[error("test functions are only allowed at module level")]
    TestFnOnlyAllowedAtModuleLevel { span: Span },
    #[error("`impl Self` for contracts is not supported")]
    SelfImplForContract { span: Span },
    #[error("Cannot attach a documentation comment to a dependency.")]
    CannotDocCommentDependency { span: Span },
    #[error("Cannot annotate a dependency.")]
    CannotAnnotateDependency { span: Span },
    #[error("Expected dependency at the beginning before any other items.")]
    ExpectedDependencyAtBeginning { span: Span },
}

impl Spanned for ConvertParseTreeError {
    fn span(&self) -> Span {
        match self {
            ConvertParseTreeError::PubUseNotSupported { span } => span.clone(),
            ConvertParseTreeError::FunctionArbitraryExpression { span } => span.clone(),
            ConvertParseTreeError::GenericsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::TupleIndexOutOfRange { span } => span.clone(),
            ConvertParseTreeError::ShlNotImplemented { span } => span.clone(),
            ConvertParseTreeError::ShrNotImplemented { span } => span.clone(),
            ConvertParseTreeError::BitXorNotImplemented { span } => span.clone(),
            ConvertParseTreeError::IntTySuffixNotSupported { span } => span.clone(),
            ConvertParseTreeError::IntLiteralOutOfRange { span } => span.clone(),
            ConvertParseTreeError::IntLiteralExpected { span } => span.clone(),
            ConvertParseTreeError::QualifiedPathRootsNotImplemented { span } => span.clone(),
            ConvertParseTreeError::CharLiteralsNotImplemented { span } => span.clone(),
            ConvertParseTreeError::HexLiteralLength { span } => span.clone(),
            ConvertParseTreeError::BinaryLiteralLength { span } => span.clone(),
            ConvertParseTreeError::U8LiteralOutOfRange { span } => span.clone(),
            ConvertParseTreeError::U16LiteralOutOfRange { span } => span.clone(),
            ConvertParseTreeError::U32LiteralOutOfRange { span } => span.clone(),
            ConvertParseTreeError::U64LiteralOutOfRange { span } => span.clone(),
            ConvertParseTreeError::SignedIntegersNotSupported { span } => span.clone(),
            ConvertParseTreeError::RefVariablesNotSupported { span } => span.clone(),
            ConvertParseTreeError::LiteralPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::ConstantPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::ConstructorPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::StructPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::WildcardPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::TuplePatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::RefPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::ConstructorPatternOneArg { span } => span.clone(),
            ConvertParseTreeError::ConstructorPatternSubPatterns { span } => span.clone(),
            ConvertParseTreeError::PathsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::FullySpecifiedTypesNotSupported { span } => span.clone(),
            ConvertParseTreeError::ContractCallerOneGenericArg { span } => span.clone(),
            ConvertParseTreeError::ContractCallerNamedTypeGenericArg { span } => span.clone(),
            ConvertParseTreeError::InvalidAttributeArgument { span, .. } => span.clone(),
            ConvertParseTreeError::ConstrainedNonExistentType { span, .. } => span.clone(),
            ConvertParseTreeError::GetStorageKeyTooManyArgs { span, .. } => span.clone(),
            ConvertParseTreeError::RecursiveType { span } => span.clone(),
            ConvertParseTreeError::DuplicateEnumVariant { span, .. } => span.clone(),
            ConvertParseTreeError::DuplicateStorageField { span, .. } => span.clone(),
            ConvertParseTreeError::DuplicateConfigurableField { span, .. } => span.clone(),
            ConvertParseTreeError::DuplicateStructField { span, .. } => span.clone(),
            ConvertParseTreeError::DuplicateParameterIdentifier { span, .. } => span.clone(),
            ConvertParseTreeError::SelfParameterNotAllowedForFreeFn { span, .. } => span.clone(),
            ConvertParseTreeError::TestFnOnlyAllowedAtModuleLevel { span } => span.clone(),
            ConvertParseTreeError::SelfImplForContract { span, .. } => span.clone(),
            ConvertParseTreeError::CannotDocCommentDependency { span } => span.clone(),
            ConvertParseTreeError::CannotAnnotateDependency { span } => span.clone(),
            ConvertParseTreeError::ExpectedDependencyAtBeginning { span } => span.clone(),
        }
    }
}
