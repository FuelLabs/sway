use crate::Ident;

/// A partially resolved type is pending further information to be typed.
/// This could be the number of bits in an integer, or it could be a generic/self type.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PartiallyResolvedType<'sc> {
    Numeric,
    SelfType,
    Generic { name: Ident<'sc> },
    NeedsType,
}

impl<'sc> PartiallyResolvedType<'sc> {
    pub(crate) fn friendly_type_str(&self) -> String {
        match self {
            PartiallyResolvedType::Generic { name } => name.primary_name.to_string(),
            PartiallyResolvedType::Numeric => "numeric".into(),
            PartiallyResolvedType::SelfType => "self".into(),
            PartiallyResolvedType::NeedsType => "needs_type".into(),
        }
    }
}
