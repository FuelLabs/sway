use super::IntegerBits;
use crate::{error::*, Ident};
use pest::Span;
/// [ResolvedType] refers to a fully qualified type that has been looked up in the namespace.
/// Type symbols are ambiguous in the beginning of compilation, as any custom symbol could be
/// an enum, struct, or generic type name. This enum is similar to [TypeInfo], except it lacks
/// the capability to be `TypeInfo::Custom`, i.e., pending this resolution of whether it is generic or a
/// known type. This allows us to ensure structurally that no unresolved types bleed into the
/// syntax tree.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ResolvedType<'sc> {
    String,
    UnsignedInteger(IntegerBits),
    Boolean,
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
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

impl<'sc> ResolvedType<'sc> {
    pub(crate) fn friendly_type_str(&self) -> String {
        use ResolvedType::*;
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
    pub(crate) fn is_convertable(
        &self,
        other: &ResolvedType<'sc>,
        debug_span: Span<'sc>,
        help_text: impl Into<String>,
    ) -> Result<Option<Warning<'sc>>, TypeError<'sc>> {
        let help_text = help_text.into();
        if *self == ResolvedType::ErrorRecovery || *other == ResolvedType::ErrorRecovery {
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
    fn numeric_cast_compat(&self, other: &ResolvedType<'sc>) -> Result<(), Warning<'sc>> {
        assert!(self.is_numeric(), other.is_numeric());
        use ResolvedType::*;
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
    fn is_numeric(&self) -> bool {
        if let ResolvedType::UnsignedInteger(_) = self {
            true
        } else {
            false
        }
    }
}
