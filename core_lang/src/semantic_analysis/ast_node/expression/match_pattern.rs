use super::*;
use crate::types::*;

#[derive(Clone, Debug)]
pub struct TypedMatchPattern<'sc> {
    pub(crate) pattern: PatternVariant<'sc>,
    pub(crate) return_type: MaybeResolvedType<'sc>,
}
