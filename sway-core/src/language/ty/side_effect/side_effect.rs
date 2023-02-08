use super::TyUseStatement;

#[derive(Clone, Debug)]
pub struct TySideEffect {
    pub side_effect: TySideEffectVariant,
}

#[derive(Clone, Debug)]
pub enum TySideEffectVariant {
    IncludeStatement,
    UseStatement(TyUseStatement),
}
