/// The calling context of a function can be used to limit a function to a
/// particular call context either internal (contract) or external (non-contract).
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum CallingContext {
    Unspecified,
    InternalOnly,
}

impl CallingContext {
    pub fn can_call(&self, other: CallingContext) -> bool {
        match other {
            CallingContext::InternalOnly => *self == CallingContext::InternalOnly,
            _ => true,
        }
    }

    // Useful for error messages, show the syntax needed in the #[context(...)] attribute.
    pub fn to_attribute_syntax(&self) -> String {
        use crate::constants::*;
        match self {
            CallingContext::Unspecified => "".to_owned(),
            CallingContext::InternalOnly => CALLING_CONTEXT_INTERNAL_ONLY_NAME.to_owned(),
        }
    }
}

impl Default for CallingContext {
    fn default() -> Self {
        CallingContext::Unspecified
    }
}

/// Utility to find the union of calling contexts.
pub fn promote_calling_context(from: CallingContext, to: CallingContext) -> CallingContext {
    match (from, to) {
        (CallingContext::Unspecified, CallingContext::InternalOnly) => CallingContext::InternalOnly,
        _otherwise => to,
    }
}
