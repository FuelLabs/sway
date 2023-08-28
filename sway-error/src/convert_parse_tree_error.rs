use sway_types::{Ident, MaybeSpanned, Span};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConvertParseTreeError {
    #[error("pub use imports are not supported")]
    PubUseNotSupported { span: Span },
    #[error("functions used in applications may not be arbitrary expressions")]
    FunctionArbitraryExpression { span: Span },
    #[error("generics are not supported here")]
    GenericsNotSupportedHere { span: Option<Span> },
    #[error("multiple generics are not supported")]
    MultipleGenericsNotSupported { span: Span },
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
    #[error("or patterns not supported in this position")]
    OrPatternsNotSupportedHere { span: Span },
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
    #[error("configurable \"{name}\" already declared")]
    DuplicateConfigurable { name: Ident, span: Span },
    #[error("Multiple configurable blocks detected in this module")]
    MultipleConfigurableBlocksInModule { span: Span },
    #[error("struct field \"{name}\" already declared")]
    DuplicateStructField { name: Ident, span: Span },
    #[error("identifier \"{name}\" bound more than once in this parameter list")]
    DuplicateParameterIdentifier { name: Ident, span: Span },
    #[error("self parameter is not allowed for {fn_kind}")]
    SelfParameterNotAllowedForFn { fn_kind: String, span: Span },
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
    #[error("Ref expressions are not supported yet.")]
    RefExprNotYetSupported { span: Span },
    #[error("Deref expressions are not supported yet.")]
    DerefExprNotYetSupported { span: Span },
    #[error("Constant requires expression.")]
    ConstantRequiresExpression { span: Span },
    #[error("Constant requires type ascription.")]
    ConstantRequiresTypeAscription { span: Span },
    #[error("Invalid value \"{value}\"")]
    InvalidCfgTargetArgValue { span: Span, value: String },
    #[error("Expected a value for the target argument")]
    ExpectedCfgTargetArgValue { span: Span },
    #[error("Invalid value \"{value}\"")]
    InvalidCfgProgramTypeArgValue { span: Span, value: String },
    #[error("Expected a value for the program_type argument")]
    ExpectedCfgProgramTypeArgValue { span: Span },
    #[error("Unexpected call path segments between qualified root and method name.")]
    UnexpectedCallPathPrefixAfterQualifiedRoot { span: Span },
}

impl MaybeSpanned for ConvertParseTreeError {
    fn try_span(&self) -> Option<Span> {
        match self {
            ConvertParseTreeError::PubUseNotSupported { span } => Some(span.clone()),
            ConvertParseTreeError::FunctionArbitraryExpression { span } => Some(span.clone()),
            ConvertParseTreeError::GenericsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::MultipleGenericsNotSupported { span } => Some(span.clone()),
            ConvertParseTreeError::TupleIndexOutOfRange { span } => Some(span.clone()),
            ConvertParseTreeError::ShlNotImplemented { span } => Some(span.clone()),
            ConvertParseTreeError::ShrNotImplemented { span } => Some(span.clone()),
            ConvertParseTreeError::BitXorNotImplemented { span } => Some(span.clone()),
            ConvertParseTreeError::IntTySuffixNotSupported { span } => Some(span.clone()),
            ConvertParseTreeError::IntLiteralOutOfRange { span } => Some(span.clone()),
            ConvertParseTreeError::IntLiteralExpected { span } => Some(span.clone()),
            ConvertParseTreeError::QualifiedPathRootsNotImplemented { span } => Some(span.clone()),
            ConvertParseTreeError::CharLiteralsNotImplemented { span } => Some(span.clone()),
            ConvertParseTreeError::HexLiteralLength { span } => Some(span.clone()),
            ConvertParseTreeError::BinaryLiteralLength { span } => Some(span.clone()),
            ConvertParseTreeError::U8LiteralOutOfRange { span } => Some(span.clone()),
            ConvertParseTreeError::U16LiteralOutOfRange { span } => Some(span.clone()),
            ConvertParseTreeError::U32LiteralOutOfRange { span } => Some(span.clone()),
            ConvertParseTreeError::U64LiteralOutOfRange { span } => Some(span.clone()),
            ConvertParseTreeError::SignedIntegersNotSupported { span } => Some(span.clone()),
            ConvertParseTreeError::RefVariablesNotSupported { span } => Some(span.clone()),
            ConvertParseTreeError::LiteralPatternsNotSupportedHere { span } => Some(span.clone()),
            ConvertParseTreeError::ConstantPatternsNotSupportedHere { span } => Some(span.clone()),
            ConvertParseTreeError::ConstructorPatternsNotSupportedHere { span } => {
                Some(span.clone())
            }
            ConvertParseTreeError::StructPatternsNotSupportedHere { span } => Some(span.clone()),
            ConvertParseTreeError::WildcardPatternsNotSupportedHere { span } => Some(span.clone()),
            ConvertParseTreeError::OrPatternsNotSupportedHere { span } => Some(span.clone()),
            ConvertParseTreeError::TuplePatternsNotSupportedHere { span } => Some(span.clone()),
            ConvertParseTreeError::RefPatternsNotSupportedHere { span } => Some(span.clone()),
            ConvertParseTreeError::ConstructorPatternOneArg { span } => Some(span.clone()),
            ConvertParseTreeError::ConstructorPatternSubPatterns { span } => Some(span.clone()),
            ConvertParseTreeError::PathsNotSupportedHere { span } => Some(span.clone()),
            ConvertParseTreeError::FullySpecifiedTypesNotSupported { span } => Some(span.clone()),
            ConvertParseTreeError::ContractCallerOneGenericArg { span } => Some(span.clone()),
            ConvertParseTreeError::ContractCallerNamedTypeGenericArg { span } => Some(span.clone()),
            ConvertParseTreeError::InvalidAttributeArgument { span, .. } => Some(span.clone()),
            ConvertParseTreeError::ConstrainedNonExistentType { span, .. } => Some(span.clone()),
            ConvertParseTreeError::GetStorageKeyTooManyArgs { span, .. } => Some(span.clone()),
            ConvertParseTreeError::RecursiveType { span } => Some(span.clone()),
            ConvertParseTreeError::DuplicateEnumVariant { span, .. } => Some(span.clone()),
            ConvertParseTreeError::DuplicateStorageField { span, .. } => Some(span.clone()),
            ConvertParseTreeError::DuplicateConfigurable { span, .. } => Some(span.clone()),
            ConvertParseTreeError::MultipleConfigurableBlocksInModule { span } => {
                Some(span.clone())
            }
            ConvertParseTreeError::DuplicateStructField { span, .. } => Some(span.clone()),
            ConvertParseTreeError::DuplicateParameterIdentifier { span, .. } => Some(span.clone()),
            ConvertParseTreeError::SelfParameterNotAllowedForFn { span, .. } => Some(span.clone()),
            ConvertParseTreeError::TestFnOnlyAllowedAtModuleLevel { span } => Some(span.clone()),
            ConvertParseTreeError::SelfImplForContract { span, .. } => Some(span.clone()),
            ConvertParseTreeError::CannotDocCommentDependency { span } => Some(span.clone()),
            ConvertParseTreeError::CannotAnnotateDependency { span } => Some(span.clone()),
            ConvertParseTreeError::ExpectedDependencyAtBeginning { span } => Some(span.clone()),
            ConvertParseTreeError::RefExprNotYetSupported { span } => Some(span.clone()),
            ConvertParseTreeError::DerefExprNotYetSupported { span } => Some(span.clone()),
            ConvertParseTreeError::ConstantRequiresExpression { span } => Some(span.clone()),
            ConvertParseTreeError::ConstantRequiresTypeAscription { span } => Some(span.clone()),
            ConvertParseTreeError::InvalidCfgTargetArgValue { span, .. } => Some(span.clone()),
            ConvertParseTreeError::ExpectedCfgTargetArgValue { span } => Some(span.clone()),
            ConvertParseTreeError::InvalidCfgProgramTypeArgValue { span, .. } => Some(span.clone()),
            ConvertParseTreeError::ExpectedCfgProgramTypeArgValue { span } => Some(span.clone()),
            ConvertParseTreeError::UnexpectedCallPathPrefixAfterQualifiedRoot { span } => {
                Some(span.clone())
            }
        }
    }
}
