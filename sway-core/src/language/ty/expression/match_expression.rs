use sway_types::Span;

use crate::{language::ty::*, type_system::*};

#[derive(Debug)]
pub(crate) struct TyMatchExpression {
    pub(crate) value_type_id: TypeId,
    pub(crate) branches: Vec<TyMatchBranch>,
    pub(crate) return_type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct TyMatchBranch {
    /// [TyExpression] of type bool that contains the condition to be used
    /// in the desugared if expression or `None` if the match arm is
    /// a catch-all arm without condition.
    /// The catch-all case needs to be distinguished later on when building
    /// the overall desugared match arm representation.
    /// That's why we return [Option] here and not an expression
    /// representing a boolean constant `true`.
    pub(crate) if_condition: Option<TyExpression>,
    /// [ty::TyCodeBlock] that includes both the match arm variable declarations
    /// that we create and the typed result from the original untyped branch result.
    pub(crate) result: TyExpression,
    #[allow(dead_code)]
    pub(crate) span: Span,
}
