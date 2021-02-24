use crate::error::{ParseResult, Warning};
use crate::{CodeBlock, ParseError, Rule};
use either::Either;
use inflector::cases::snakecase::is_snake_case;
use pest::iterators::Pair;
use pest::Span;
/// Type information without an associated value, used for type inferencing and definition.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo<'sc> {
    String,
    UnsignedInteger(IntegerBits),
    Boolean,
    Generic { name: &'sc str },
    Unit,
    SelfType,
    Byte,
    Byte32,
}
#[derive(Debug, Clone, PartialEq)]
pub enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
    OneTwentyEight,
}

impl<'sc> TypeInfo<'sc> {
    pub(crate) fn parse_from_pair(input: Pair<'sc, Rule>) -> Result<Self, ParseError<'sc>> {
        let mut r#type = input.into_inner();
        Self::parse_from_pair_inner(r#type.next().unwrap())
    }
    pub(crate) fn parse_from_pair_inner(input: Pair<'sc, Rule>) -> Result<Self, ParseError<'sc>> {
        Ok(match input.as_str() {
            "u8" => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            "u16" => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
            "u32" => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            "u64" => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            "u128" => TypeInfo::UnsignedInteger(IntegerBits::OneTwentyEight),
            "bool" => TypeInfo::Boolean,
            "string" => TypeInfo::String,
            "unit" => TypeInfo::Unit,
            other => TypeInfo::Generic { name: other },
        })
    }

    pub(crate) fn is_convertable(
        &self,
        other: &'sc TypeInfo<'sc>,
        debug_span: Span<'sc>,
    ) -> Result<Option<Warning>, crate::semantics::error::TypeError> {
        use crate::semantics::error::TypeError;
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
                expected: self.friendly_type_str(),
                received: other.friendly_type_str(),
                span: debug_span,
            })
        }
    }

    fn numeric_cast_compat(&self, other: &'sc TypeInfo<'sc>) -> Result<(), Warning<'sc>> {
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
            Generic { name } => format!("<Generic {}>", name),
            Unit => "()".into(),
            SelfType => "Self".into(),
            Byte => "byte".into(),
            Byte32 => "byte32".into(),
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
