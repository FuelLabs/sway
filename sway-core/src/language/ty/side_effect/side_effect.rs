use super::{TyIncludeStatement, TyUseStatement};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TySideEffect {
    pub side_effect: TySideEffectVariant,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TySideEffectVariant {
    IncludeStatement(TyIncludeStatement),
    UseStatement(TyUseStatement),
}
