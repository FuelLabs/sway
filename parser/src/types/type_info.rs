use crate::error::*;
use crate::{parse_tree::Ident, Rule};
use pest::iterators::Pair;
use pest::Span;

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
    Generic {
        name: Ident<'sc>,
    },
    Unit,
    SelfType,
    Byte,
    Byte32,
    Struct {
        name: Ident<'sc>,
    },
    Enum {
        name: Ident<'sc>,
    },
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
    pub(crate) fn to_resolved(&self) -> ResolvedType {
        match self {
            TypeInfo::Generic { .. } | TypeInfo::Struct { .. } | TypeInfo::Enum { .. } | TypeInfo::Custom { .. } => panic!("Invalid use of `to_resolved`. See documentation of [TypeInfo::to_resolved] for more details."),
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
                "Self" => TypeInfo::SelfType,
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

    pub(crate) fn is_convertable(
        &self,
        other: &TypeInfo<'sc>,
        debug_span: Span<'sc>,
        help_text: impl Into<String>,
    ) -> Result<Option<Warning<'sc>>, TypeError<'sc>> {
        let help_text = help_text.into();
        if *self == TypeInfo::ErrorRecovery || *other == TypeInfo::ErrorRecovery {
            return Ok(None);
        }
        // TODO  actually check more advanced conversion rules like upcasting vs downcasting
        // numbers, emit warnings for loss of precision
        if self == other {
            Ok(None)
        } else if self.is_numeric() && other.is_numeric() {
            // check numeric castability
            match self.numeric_cast_compat(other) {
                Ok(()) => Ok(None),
                Err(warn) => Ok(Some(warn)),
            }
        } else {
            Err(TypeError::MismatchedType {
                expected: other.friendly_type_str(),
                received: self.friendly_type_str(),
                help_text,
                span: debug_span,
            })
        }
    }

    fn numeric_cast_compat(&self, other: &TypeInfo<'sc>) -> Result<(), Warning<'sc>> {
        assert!(self.is_numeric(), other.is_numeric());
        use TypeInfo::*;
        // if this is a downcast, warn for loss of precision. if upcast, then no warning.
        match self {
            UnsignedInteger(IntegerBits::Eight) => Ok(()),
            UnsignedInteger(IntegerBits::Sixteen) => match other {
                UnsignedInteger(IntegerBits::Eight) => Err(Warning::LossOfPrecision {
                    initial_type: self.clone(),
                    cast_to: other.clone(),
                }),
                UnsignedInteger(_) => Ok(()),
                _ => unreachable!(),
            },
            UnsignedInteger(IntegerBits::ThirtyTwo) => match other {
                UnsignedInteger(IntegerBits::Eight) | UnsignedInteger(IntegerBits::Sixteen) => {
                    Err(Warning::LossOfPrecision {
                        initial_type: self.clone(),
                        cast_to: other.clone(),
                    })
                }
                UnsignedInteger(_) => Ok(()),
                _ => unreachable!(),
            },
            UnsignedInteger(IntegerBits::SixtyFour) => match other {
                UnsignedInteger(IntegerBits::Eight)
                | UnsignedInteger(IntegerBits::Sixteen)
                | UnsignedInteger(IntegerBits::ThirtyTwo) => Err(Warning::LossOfPrecision {
                    initial_type: self.clone(),
                    cast_to: other.clone(),
                }),
                _ => Ok(()),
            },
            UnsignedInteger(IntegerBits::OneTwentyEight) => match other {
                UnsignedInteger(IntegerBits::OneTwentyEight) => Ok(()),
                _ => Err(Warning::LossOfPrecision {
                    initial_type: self.clone(),
                    cast_to: other.clone(),
                }),
            },
            _ => unreachable!(),
        }
    }

    pub(crate) fn friendly_type_str(&self) -> String {
        use TypeInfo::*;
        match self {
            String => "String".into(),
            UnsignedInteger(bits) => {
                use IntegerBits::*;
                match bits {
                    Eight => "u8",
                    Sixteen => "u16",
                    ThirtyTwo => "u32",
                    SixtyFour => "u64",
                    OneTwentyEight => "u128",
                }
                .into()
            }
            Boolean => "bool".into(),
            Generic { name } => format!("generic {}", name.primary_name),
            Custom { name } => format!("unknown {}", name.primary_name),
            Unit => "()".into(),
            SelfType => "Self".into(),
            Byte => "byte".into(),
            Byte32 => "byte32".into(),
            Struct {
                name: Ident { primary_name, .. },
                ..
            } => format!("struct {}", primary_name),
            Enum {
                name: Ident { primary_name, .. },
                ..
            } => format!("enum {}", primary_name),
            ErrorRecovery => "\"unknown due to error\"".into(),
        }
    }
    fn is_numeric(&self) -> bool {
        if let TypeInfo::UnsignedInteger(_) = self {
            true
        } else {
            false
        }
    }
}
