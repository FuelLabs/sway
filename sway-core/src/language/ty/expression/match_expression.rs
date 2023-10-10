use sway_types::{Span, Ident};

use crate::{language::ty::*, type_system::*};

/// [TyExpression] of type bool that contains the condition to be used
/// in the desugared if expression or `None` if the match arm is
/// a catch-all arm without condition.
/// The catch-all case needs to be distinguished later on when building
/// the overall desugared match arm representation.
/// That's why we return [Option] here and not an expression
/// representing a boolean constant `true`.
/// E.g., a condition might look like:
/// `__match_val_1.x == 11 && __match_val_1.y == 22 || __match_val_1.x == 33 && __match_val_1.y == 44`
pub(crate) type MatchIfCondition = Option<TyExpression>;

/// [TyExpression]s of the form `let <ident> = <expression>` where
/// `<ident>` is a name of a generated `__or_variant_vars` variable
/// and `<expression>` is an `if-else` expression that returns
/// an Option of a tuple containing values of each of the variables
/// declared in an OR match pattern.
pub(crate) type MatchOrVariantVars = Vec<(Ident, TyExpression)>;

#[derive(Debug)]
pub(crate) struct TyMatchExpression {
    pub(crate) value_type_id: TypeId,
    pub(crate) branches: Vec<TyMatchBranch>,
    pub(crate) return_type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) struct TyMatchBranch {
    pub(crate) or_variant_vars: MatchOrVariantVars,
    /// A boolean expression that represents the total match arm requirement,
    /// or `None` if the match arm is a catch-all arm.
    pub(crate) if_condition: MatchIfCondition,
    /// The resulting [ty::TyCodeBlock] that includes the match arm variable declarations
    /// and the typed result from the original untyped branch result.
    pub(crate) result: TyExpression,
    #[allow(dead_code)]
    pub(crate) span: Span,
}
