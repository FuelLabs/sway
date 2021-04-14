use crate::error::*;
use crate::{Ident, Rule};
use pest::iterators::Pair;

use super::ResolvedType;

/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo<'sc> {
    String,
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
    // used for recovering from errors in the ast
    ErrorRecovery,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
    OneTwentyEight,
}

impl<'sc> TypeInfo<'sc> {
    /// This is a shortcut function. It should only be called as a convenience method in match
    /// statements resolving types when it has already been verified that this type is _not_
    /// a custom (enum, struct, user-defined) or generic type.
    /// This function just passes all the trivial types through to a [ResolvedType].
    pub(crate) fn to_resolved(&self) -> ResolvedType<'sc> {
        match self {
            TypeInfo::Custom { .. } => panic!("Invalid use of `to_resolved`. See documentation of [TypeInfo::to_resolved] for more details."),
            TypeInfo::Boolean => ResolvedType::Boolean,
            TypeInfo::String => ResolvedType::String,
            TypeInfo::UnsignedInteger(bits) => ResolvedType::UnsignedInteger(*bits),
            TypeInfo::Unit => ResolvedType::Unit,
            TypeInfo::SelfType => ResolvedType::SelfType,
            TypeInfo::Byte => ResolvedType::Byte,
            TypeInfo::Byte32 => ResolvedType::Byte32,
            TypeInfo::ErrorRecovery => ResolvedType::ErrorRecovery

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
                "u128" => TypeInfo::UnsignedInteger(IntegerBits::OneTwentyEight),
                "bool" => TypeInfo::Boolean,
                "string" => TypeInfo::String,
                "unit" => TypeInfo::Unit,
                "byte" => TypeInfo::Byte,
                "Self" | "self" => TypeInfo::SelfType,
                "()" => TypeInfo::Unit,
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
