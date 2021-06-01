use super::{MaybeResolvedType, ResolvedType};
use crate::error::*;
use crate::types::PartiallyResolvedType;
use crate::{Ident, Rule};
use pest::iterators::Pair;
use std::iter::FromIterator;

/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo<'sc> {
    Str(u64),
    UnsignedInteger(IntegerBits),
    Boolean,
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
    Custom {
        name: Ident<'sc>,
    },
    Unit,
    SelfType,
    Byte,
    Byte32,
    /// This means that specific type of a number is not yet known. It will be
    /// determined via inference at a later time.
    Numeric,
    // used for recovering from errors in the ast
    ErrorRecovery,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

impl<'sc> TypeInfo<'sc> {
    /// This is a shortcut function. It should only be called as a convenience method in match
    /// statements resolving types when it has already been verified that this type is _not_
    /// a custom (enum, struct, user-defined) or generic type.
    /// This function just passes all the trivial types through to a [ResolvedType].
    pub(crate) fn to_resolved(&self) -> MaybeResolvedType<'sc> {
        match self {
            TypeInfo::Custom { .. } | TypeInfo::SelfType => panic!(
                "Invalid use of `to_resolved`. See documentation of [TypeInfo::to_resolved] for \
                 more details."
            ),
            TypeInfo::Boolean => MaybeResolvedType::Resolved(ResolvedType::Boolean),
            TypeInfo::Str(len) => MaybeResolvedType::Resolved(ResolvedType::Str(*len)),
            TypeInfo::UnsignedInteger(bits) => {
                MaybeResolvedType::Resolved(ResolvedType::UnsignedInteger(*bits))
            }
            TypeInfo::Numeric => MaybeResolvedType::Partial(PartiallyResolvedType::Numeric),
            TypeInfo::Unit => MaybeResolvedType::Resolved(ResolvedType::Unit),
            TypeInfo::Byte => MaybeResolvedType::Resolved(ResolvedType::Byte),
            TypeInfo::Byte32 => MaybeResolvedType::Resolved(ResolvedType::Byte32),
            TypeInfo::ErrorRecovery => MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery),
        }
    }
    pub(crate) fn parse_from_pair(input: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut r#type = input.into_inner();
        Self::parse_from_pair_inner(r#type.next().unwrap())
    }
    pub(crate) fn parse_from_pair_inner(input: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        ok(
            match input.as_str().trim() {
                "u8" => TypeInfo::UnsignedInteger(IntegerBits::Eight),
                "u16" => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
                "u32" => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                "u64" => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                "bool" => TypeInfo::Boolean,
                "unit" => TypeInfo::Unit,
                "byte" => TypeInfo::Byte,
                "byte32" => TypeInfo::Byte32,
                "Self" | "self" => TypeInfo::SelfType,
                "()" => TypeInfo::Unit,
                a if a.contains("str[") => type_check!(
                    parse_str_type(a, input.as_span()),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                _other => TypeInfo::Custom {
                    name: eval!(
                        Ident::parse_from_pair,
                        warnings,
                        errors,
                        input,
                        return err(warnings, errors)
                    ),
                },
            },
            warnings,
            errors,
        )
    }
}

fn parse_str_type<'sc>(raw: &'sc str, span: pest::Span<'sc>) -> CompileResult<'sc, TypeInfo<'sc>> {
    if raw.starts_with("str[") {
        let mut rest = raw.split_at("str[".len()).1.chars().collect::<Vec<_>>();
        if let Some(']') = rest.pop() {
            if let Ok(num) = String::from_iter(rest).parse() {
                return ok(TypeInfo::Str(num), vec![], vec![]);
            }
        }
        return err(vec![], vec![CompileError::InvalidStrType { raw, span }]);
    }
    return err(vec![], vec![CompileError::UnknownType { span }]);
}

#[test]
fn test_str_parse() {
    match parse_str_type("str[20]", pest::Span::new("", 0, 0).unwrap()) {
        CompileResult::Ok { value, .. } if value == TypeInfo::Str(20) => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("str[]", pest::Span::new("", 0, 0).unwrap()) {
        CompileResult::Err { .. } => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("str[ab]", pest::Span::new("", 0, 0).unwrap()) {
        CompileResult::Err { .. } => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("str [ab]", pest::Span::new("", 0, 0).unwrap()) {
        CompileResult::Err { .. } => (),
        _ => panic!("failed test"),
    }

    match parse_str_type("not even a str[ type", pest::Span::new("", 0, 0).unwrap()) {
        CompileResult::Err { .. } => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("", pest::Span::new("", 0, 0).unwrap()) {
        CompileResult::Err { .. } => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("20", pest::Span::new("", 0, 0).unwrap()) {
        CompileResult::Err { .. } => (),
        _ => panic!("failed test"),
    }
    match parse_str_type("[20]", pest::Span::new("", 0, 0).unwrap()) {
        CompileResult::Err { .. } => (),
        _ => panic!("failed test"),
    }
}
