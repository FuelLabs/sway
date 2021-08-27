use super::*;
use crate::types::*;
use crate::Ident;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct TypedMatchBranch<'sc> {
    pub(crate) patterns: Vec<(Ident<'sc>, MaybeResolvedType<'sc>)>,
    pub(crate) result: TypedExpression<'sc>,
}
