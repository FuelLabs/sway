use super::{TyIncludeStatement, TyUseStatement};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TySideEffect {
    pub side_effect: TySideEffectVariant,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TySideEffectVariant {
    IncludeStatement(TyIncludeStatement),
    UseStatement(TyUseStatement),
}
