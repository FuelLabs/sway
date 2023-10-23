use sway_types::{Ident, Span};

use crate::{language::ty::*, type_system::*};

/// [TyExpression] of type bool that contains the condition to be used
/// in the desugared if expression or `None` if the match arm is
/// a catch-all arm without condition.
/// E.g., a condition might look like:
/// `__matched_value_1.x == 11 && __matched_value_1.y == 22 || __matched_value_1.x == 33 && __matched_value_1.y == 44`
pub(crate) type MatchBranchCondition = Option<TyExpression>;

/// [TyExpression]s of the form `let <ident> = <expression>` where
/// `<ident>` is a name of a generated  variable that holds the
/// index of the matched OR variant.
/// `<expression>` is an `if-else` expression that returns
/// the 1-based index of the matched OR variant or zero
/// if non of the variants match.
pub(crate) type MatchedOrVariantIndexVars = Vec<(Ident, TyExpression)>;

#[derive(Debug)]
pub(crate) struct TyMatchExpression {
    pub(crate) value_type_id: TypeId,
    pub(crate) branches: Vec<TyMatchBranch>,
    pub(crate) return_type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct TyMatchBranch {
    /// Declarations of the variables that hold the 1-based index
    /// of a matched OR variant or zero if non of the variants match.
    pub(crate) matched_or_variant_index_vars: MatchedOrVariantIndexVars,
    /// A boolean expression that represents the total match arm requirement,
    /// or `None` if the match arm is a catch-all arm.
    pub(crate) condition: MatchBranchCondition,
    /// The resulting [crate::ty::TyCodeBlock] that includes the match arm variable declarations
    /// and the typed result from the original untyped branch result.
    pub(crate) result: TyExpression,
    #[allow(dead_code)]
    pub(crate) span: Span,
}
