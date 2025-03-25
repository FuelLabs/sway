use sway_types::{Ident, IdentUnique, Span, Spanned};
use thiserror::Error;

use crate::formatting::{
    a_or_an, num_to_str, num_to_str_or_none, plural_s, sequence_to_str, Enclosing,
};

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConvertParseTreeError {
    #[error("Imports without items are not supported")]
    ImportsWithoutItemsNotSupported { span: Span },
    #[error("functions used in applications may not be arbitrary expressions")]
    FunctionArbitraryExpression { span: Span },
    #[error("generics are not supported here")]
    GenericsNotSupportedHere { span: Span },
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
    #[error("`impl Self` for contracts is not supported")]
    SelfImplForContract { span: Span },
    #[error("Expected module at the beginning before any other items.")]
    ExpectedModuleAtBeginning { span: Span },
    #[error("Constant requires expression.")]
    ConstantRequiresExpression { span: Span },
    #[error("Constant requires type ascription.")]
    ConstantRequiresTypeAscription { span: Span },
    #[error("Unknown type name \"self\". A self type with a similar name exists (notice the capitalization): `Self`")]
    UnknownTypeNameSelf { span: Span },
    #[error("{}", match get_attribute_type(attribute) {
        AttributeType::InnerDocComment => format!("Inner doc comment (`//!`) cannot document {}{target_friendly_name}.", a_or_an(&target_friendly_name)),
        AttributeType::OuterDocComment => format!("Outer doc comment (`///`) cannot document {}{target_friendly_name}.", a_or_an(&target_friendly_name)),
        AttributeType::Attribute => format!("\"{attribute}\" attribute cannot annotate {}{target_friendly_name}.", a_or_an(&target_friendly_name)),
    })]
    InvalidAttributeTarget {
        span: Span,
        attribute: Ident,
        target_friendly_name: &'static str,
        can_only_annotate_help: Vec<&'static str>,
    },
    #[error("\"{last_occurrence}\" attribute can be applied only once, but is applied {} times.", num_to_str(previous_occurrences.len() + 1))]
    InvalidAttributeMultiplicity {
        last_occurrence: IdentUnique,
        previous_occurrences: Vec<IdentUnique>,
    },
    #[error("\"{attribute}\" attribute must {}, but has {}.", get_expected_attributes_args_multiplicity_msg(args_multiplicity), num_to_str_or_none(*num_of_args))]
    InvalidAttributeArgsMultiplicity {
        span: Span,
        attribute: Ident,
        args_multiplicity: (usize, usize),
        num_of_args: usize,
    },
    #[error(
        "\"{arg}\" is an invalid argument for attribute \"{attribute}\". Valid arguments are: {}.",
        sequence_to_str(expected_args, Enclosing::DoubleQuote, usize::MAX)
    )]
    InvalidAttributeArg {
        attribute: Ident,
        arg: IdentUnique,
        expected_args: Vec<&'static str>,
    },
    #[error("\"{arg}\" argument of the attribute \"{attribute}\" must {}have a value.",
        match value_span {
            Some(_) => "not ",
            None => "",
        }
    )]
    InvalidAttributeArgExpectsValue {
        attribute: Ident,
        arg: IdentUnique,
        value_span: Option<Span>,
    },
    #[error("\"{arg}\" argument must have a value of type \"{expected_type}\".")]
    InvalidAttributeArgValueType {
        span: Span,
        arg: Ident,
        expected_type: &'static str,
        received_type: &'static str,
    },
    #[error("{} is an invalid value for argument \"{arg}\". Valid values are: {}.", span.as_str(), sequence_to_str(expected_values, Enclosing::DoubleQuote, usize::MAX))]
    InvalidAttributeArgValue {
        span: Span,
        arg: Ident,
        expected_values: Vec<&'static str>,
    },
}

pub(crate) enum AttributeType {
    /// `//!`.
    InnerDocComment,
    /// `///`.
    OuterDocComment,
    /// `#[attribute]` or `#![attribute]`.
    Attribute,
}

pub(crate) fn get_attribute_type(attribute: &Ident) -> AttributeType {
    // The doc-comment attribute name has the span that
    // points to the actual comment line.
    // Other attributes have spans that point to the actual
    // attribute name.
    let span = attribute.span();
    let attribute = span.as_str();
    if attribute.starts_with("//!") {
        AttributeType::InnerDocComment
    } else if attribute.starts_with("///") {
        AttributeType::OuterDocComment
    } else {
        AttributeType::Attribute
    }
}

pub(crate) fn get_expected_attributes_args_multiplicity_msg(
    args_multiplicity: &(usize, usize),
) -> String {
    match *args_multiplicity {
        (0, 0) => "not have any arguments".to_string(),
        (min, max) if min == max => format!("have exactly {} argument{}", num_to_str(min), plural_s(min)),
        (min, max) if min == max - 1 => format!("have {} or {} argument{}", num_to_str_or_none(min), num_to_str(max), plural_s(max)),
        (0, max) if max != usize::MAX => format!("have at most {} argument{}", num_to_str(max), plural_s(max)), 
        (min, usize::MAX) if min != usize::MIN => format!("have at least {} argument{}", num_to_str(min), plural_s(min)), 
        (min, max) if max != usize::MAX => format!("have between {} and {} arguments", num_to_str(min), num_to_str(max)),
        (0, usize::MAX) => unreachable!("if any number of arguments are accepted the `InvalidAttributeArgsMultiplicity` error cannot occur"),
        _ => unreachable!("`min` is `always` less than or equal to `max` and all combinations are already covered"),
    }
}

impl Spanned for ConvertParseTreeError {
    fn span(&self) -> Span {
        match self {
            ConvertParseTreeError::ImportsWithoutItemsNotSupported { span } => span.clone(),
            ConvertParseTreeError::FunctionArbitraryExpression { span } => span.clone(),
            ConvertParseTreeError::GenericsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::MultipleGenericsNotSupported { span } => span.clone(),
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
            ConvertParseTreeError::OrPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::TuplePatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::RefPatternsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::ConstructorPatternOneArg { span } => span.clone(),
            ConvertParseTreeError::ConstructorPatternSubPatterns { span } => span.clone(),
            ConvertParseTreeError::PathsNotSupportedHere { span } => span.clone(),
            ConvertParseTreeError::FullySpecifiedTypesNotSupported { span } => span.clone(),
            ConvertParseTreeError::ContractCallerOneGenericArg { span } => span.clone(),
            ConvertParseTreeError::ContractCallerNamedTypeGenericArg { span } => span.clone(),
            ConvertParseTreeError::ConstrainedNonExistentType { span, .. } => span.clone(),
            ConvertParseTreeError::GetStorageKeyTooManyArgs { span, .. } => span.clone(),
            ConvertParseTreeError::RecursiveType { span } => span.clone(),
            ConvertParseTreeError::DuplicateEnumVariant { span, .. } => span.clone(),
            ConvertParseTreeError::DuplicateStorageField { span, .. } => span.clone(),
            ConvertParseTreeError::DuplicateConfigurable { span, .. } => span.clone(),
            ConvertParseTreeError::MultipleConfigurableBlocksInModule { span } => span.clone(),
            ConvertParseTreeError::DuplicateStructField { span, .. } => span.clone(),
            ConvertParseTreeError::DuplicateParameterIdentifier { span, .. } => span.clone(),
            ConvertParseTreeError::SelfParameterNotAllowedForFn { span, .. } => span.clone(),
            ConvertParseTreeError::SelfImplForContract { span, .. } => span.clone(),
            ConvertParseTreeError::ExpectedModuleAtBeginning { span } => span.clone(),
            ConvertParseTreeError::ConstantRequiresExpression { span } => span.clone(),
            ConvertParseTreeError::ConstantRequiresTypeAscription { span } => span.clone(),
            ConvertParseTreeError::UnknownTypeNameSelf { span } => span.clone(),
            ConvertParseTreeError::InvalidAttributeTarget { span, .. } => span.clone(),
            ConvertParseTreeError::InvalidAttributeMultiplicity {
                last_occurrence: last_attribute,
                ..
            } => last_attribute.span(),
            ConvertParseTreeError::InvalidAttributeArgsMultiplicity { span, .. } => span.clone(),
            ConvertParseTreeError::InvalidAttributeArg { arg, .. } => arg.span(),
            ConvertParseTreeError::InvalidAttributeArgExpectsValue { arg, .. } => arg.span(),
            ConvertParseTreeError::InvalidAttributeArgValueType { span, .. } => span.clone(),
            ConvertParseTreeError::InvalidAttributeArgValue { span, .. } => span.clone(),
        }
    }
}
