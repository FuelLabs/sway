use super::IntegerBits;
use super::{PartiallyResolvedType, ResolvedType};
use crate::error::*;
use crate::span::Span;

/// [ResolvedType] refers to a fully qualified type that has been looked up in the namespace.
/// Type symbols are ambiguous in the beginning of compilation, as any custom symbol could be
/// an enum, struct, or generic type name. This enum is similar to [TypeInfo], except it lacks
/// the capability to be `TypeInfo::Custom`, i.e., pending this resolution of whether it is generic
/// or a known type. This allows us to ensure structurally that no unresolved types bleed into the
/// syntax tree.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MaybeResolvedType<'sc> {
    Resolved(ResolvedType<'sc>),
    Partial(PartiallyResolvedType<'sc>),
}

impl Default for MaybeResolvedType<'_> {
    fn default() -> Self {
        MaybeResolvedType::Resolved(ResolvedType::Unit)
    }
}

impl<'sc> MaybeResolvedType<'sc> {
    pub(crate) fn to_selector_name(
        &self,
        error_msg_span: &Span<'sc>,
    ) -> CompileResult<'sc, String> {
        match self {
            MaybeResolvedType::Resolved(r) => r.to_selector_name(error_msg_span),
            _ => {
                return err(
                    vec![],
                    vec![CompileError::InvalidAbiType {
                        span: error_msg_span.clone(),
                    }],
                )
            }
        }
    }
    pub(crate) fn is_copy_type(&self) -> bool {
        match self {
            MaybeResolvedType::Resolved(ty) => match ty {
                ResolvedType::UnsignedInteger(_)
                | ResolvedType::Boolean
                | ResolvedType::Unit
                | ResolvedType::Byte => true,
                _ => false,
            },
            _ => false,
        }
    }
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
            (_, MaybeResolvedType::Partial(PartiallyResolvedType::NeedsType)) => Ok(None),
            (MaybeResolvedType::Partial(PartiallyResolvedType::NeedsType), _) => Ok(None),
            _ => Err(TypeError::MismatchedType {
                expected: other.friendly_type_str(),
                received: self.friendly_type_str(),
                help_text,
                span: debug_span,
            }),
        }
    }
    /// Force this type into a [ResolvedType]. This returns an error if the type is not resolvable.
    pub(crate) fn force_resolution(
        &self,
        self_type: &MaybeResolvedType<'sc>,
        debug_span: &Span<'sc>,
    ) -> CompileResult<'sc, ResolvedType<'sc>> {
        ok(
            match (self, self_type) {
                (MaybeResolvedType::Resolved(r), _) => r.clone(),
                (MaybeResolvedType::Partial(PartiallyResolvedType::Numeric), _) => {
                    ResolvedType::UnsignedInteger(IntegerBits::SixtyFour)
                }
                (
                    MaybeResolvedType::Partial(PartiallyResolvedType::SelfType),
                    MaybeResolvedType::Resolved(r),
                ) => r.clone(),
                _ => {
                    return err(
                        vec![],
                        vec![CompileError::TypeMustBeKnown {
                            ty: self.friendly_type_str(),
                            span: debug_span.clone(),
                        }],
                    )
                }
            },
            vec![],
            vec![],
        )
    }
}
