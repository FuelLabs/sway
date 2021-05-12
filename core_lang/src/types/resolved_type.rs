use super::IntegerBits;
use crate::{error::*, semantic_analysis::ast_node::TypedStructField, Ident};
use pest::Span;
/// [ResolvedType] refers to a fully qualified type that has been looked up in the namespace.
/// Type symbols are ambiguous in the beginning of compilation, as any custom symbol could be
/// an enum, struct, or generic type name. This enum is similar to [TypeInfo], except it lacks
/// the capability to be `TypeInfo::Custom`, i.e., pending this resolution of whether it is generic or a
/// known type. This allows us to ensure structurally that no unresolved types bleed into the
/// syntax tree.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MaybeResolvedType<'sc> {
    Resolved(ResolvedType<'sc>),
    Partial(PartiallyResolvedType<'sc>),
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ResolvedType<'sc> {
    String,
    UnsignedInteger(IntegerBits),
    Boolean,
    Unit,
    Byte,
    Byte32,
    Struct {
        name: Ident<'sc>,
        fields: Vec<TypedStructField<'sc>>,
    },
    Enum {
        name: Ident<'sc>,
        variant_types: Vec<ResolvedType<'sc>>,
    },
    /// Represents the contract's type as a whole. Used for implementing
    /// traits on the contract itself, to enforce a specific type of ABI.
    Contract,
    // used for recovering from errors in the ast
    ErrorRecovery,
}
/// A partially resolved type is pending further information to be typed.
/// This could be the number of bits in an integer, or it could be a generic/self type.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PartiallyResolvedType<'sc> {
    Numeric,
    SelfType,
    Generic { name: Ident<'sc> },
}

impl Default for MaybeResolvedType<'_> {
    fn default() -> Self {
        MaybeResolvedType::Resolved(ResolvedType::Unit)
    }
}

impl<'sc> MaybeResolvedType<'sc> {
    pub(crate) fn friendly_type_str(&self) -> String {
        match self {
            MaybeResolvedType::Partial(ty) => ty.friendly_type_str(),
            MaybeResolvedType::Resolved(ty) => ty.friendly_type_str(),
        }
    }
    /// Whether or not this potentially resolved type is a numeric type.
    fn is_numeric(&self) -> bool {
        match self {
            MaybeResolvedType::Resolved(x) => x.is_numeric(),
            MaybeResolvedType::Partial(p) => p == &PartiallyResolvedType::Numeric,
        }
    }

    fn numeric_cast_compat(&self, other: &MaybeResolvedType<'sc>) -> Result<(), Warning<'sc>> {
        assert_eq!(self.is_numeric(), other.is_numeric());
        match (self, other) {
            (MaybeResolvedType::Resolved(ref r), &MaybeResolvedType::Resolved(ref r_2)) => {
                r.numeric_cast_compat(&r_2)
            }
            // because we know `p` and `r` are numeric, this is safe
            (MaybeResolvedType::Partial(_p), MaybeResolvedType::Resolved(_r)) => Ok(()),
            // because we know `p` and `r` are numeric, this is safe
            (MaybeResolvedType::Resolved(_r), MaybeResolvedType::Partial(_p)) => Ok(()),
            (MaybeResolvedType::Partial(_p), MaybeResolvedType::Partial(_p2)) => Ok(()),
        }
    }

    pub(crate) fn is_convertible(
        &self,
        other: &Self,
        debug_span: Span<'sc>,
        help_text: impl Into<String>,
    ) -> Result<Option<Warning<'sc>>, TypeError<'sc>> {
        let help_text = help_text.into();
        match (self, other) {
            (s, o) if s.is_numeric() && o.is_numeric() => match s.numeric_cast_compat(o) {
                Ok(()) => Ok(None),
                Err(warn) => Ok(Some(warn)),
            },
            (MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery), _) => Ok(None),
            (_, MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery)) => Ok(None),
            (MaybeResolvedType::Resolved(r), MaybeResolvedType::Resolved(r2)) if r == r2 => {
                Ok(None)
            }
            _ => Err(TypeError::MismatchedType {
                expected: other.friendly_type_str(),
                received: self.friendly_type_str(),
                help_text,
                span: debug_span,
            }),
        }
    }
}
impl<'sc> PartiallyResolvedType<'sc> {
    pub(crate) fn friendly_type_str(&self) -> String {
        match self {
            PartiallyResolvedType::Generic { name } => format!("generic {}", name.primary_name),
            PartiallyResolvedType::Numeric => "numeric".into(),
            PartiallyResolvedType::SelfType => "self".into(),
        }
    }
}

impl<'sc> ResolvedType<'sc> {
    fn numeric_cast_compat(&self, other: &ResolvedType<'sc>) -> Result<(), Warning<'sc>> {
        assert_eq!(self.is_numeric(), other.is_numeric());
        use ResolvedType::*;
        // if this is a downcast, warn for loss of precision. if upcast, then no warning.
        match self {
            UnsignedInteger(IntegerBits::Eight) => Ok(()),
            UnsignedInteger(IntegerBits::Sixteen) => match other {
                UnsignedInteger(IntegerBits::Eight) => Err(Warning::LossOfPrecision {
                    initial_type: MaybeResolvedType::Resolved(self.clone()),
                    cast_to: MaybeResolvedType::Resolved(other.clone()),
                }),
                UnsignedInteger(_) => Ok(()),
                _ => unreachable!(),
            },
            UnsignedInteger(IntegerBits::ThirtyTwo) => match other {
                UnsignedInteger(IntegerBits::Eight) | UnsignedInteger(IntegerBits::Sixteen) => {
                    Err(Warning::LossOfPrecision {
                        initial_type: MaybeResolvedType::Resolved(self.clone()),
                        cast_to: MaybeResolvedType::Resolved(other.clone()),
                    })
                }
                UnsignedInteger(_) => Ok(()),
                _ => unreachable!(),
            },
            UnsignedInteger(IntegerBits::SixtyFour) => match other {
                UnsignedInteger(IntegerBits::Eight)
                | UnsignedInteger(IntegerBits::Sixteen)
                | UnsignedInteger(IntegerBits::ThirtyTwo) => Err(Warning::LossOfPrecision {
                    initial_type: MaybeResolvedType::Resolved(self.clone()),
                    cast_to: MaybeResolvedType::Resolved(other.clone()),
                }),
                _ => Ok(()),
            },
            _ => unreachable!(),
        }
    }
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
                }
                .into()
            }
            Boolean => "bool".into(),

            Unit => "()".into(),
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
            Contract => "contract".into(),
            ErrorRecovery => "\"unknown due to error\"".into(),
        }
    }
    pub(crate) fn is_convertible(
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

    /// Calculates the stack size of this type, to be used when allocating stack memory for it.
    /// This is _in words_!
    pub(crate) fn stack_size_of(&self) -> u64 {
        match self {
            // the pointer to the beginning of the string is 64 bits
            ResolvedType::String => 1,
            // Since things are unpacked, all unsigned integers are 64 bits.....for now
            ResolvedType::UnsignedInteger(_) => 1,
            ResolvedType::Boolean => 1,
            ResolvedType::Unit => 0,
            ResolvedType::Byte => 1,
            ResolvedType::Byte32 => 4,
            ResolvedType::Enum { variant_types, .. } => {
                // the size of an enum is one word (for the tag) plus the maximum size
                // of any individual variant
                1 + variant_types
                    .into_iter()
                    .map(|x| x.stack_size_of())
                    .max()
                    .unwrap()
            }
            ResolvedType::Struct { fields, .. } => fields
                .iter()
                .fold(0, |acc, x| acc + x.r#type.stack_size_of()),
            ResolvedType::Contract => unreachable!("contract types are never instantiated"),
            ResolvedType::ErrorRecovery => unreachable!(),
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
