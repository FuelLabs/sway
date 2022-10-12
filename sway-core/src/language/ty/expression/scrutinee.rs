use sway_types::{Ident, Span};

use crate::{language::*, semantic_analysis::TyEnumVariant, type_system::*};

#[derive(Debug, Clone)]
pub(crate) struct TyScrutinee {
    pub(crate) variant: TyScrutineeVariant,
    pub(crate) type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub(crate) enum TyScrutineeVariant {
    CatchAll,
    Literal(Literal),
    Variable(Ident),
    Constant(Ident, Literal, TypeId),
    StructScrutinee(Ident, Vec<TyStructScrutineeField>),
    #[allow(dead_code)]
    EnumScrutinee {
        call_path: CallPath,
        variant: TyEnumVariant,
        value: Box<TyScrutinee>,
    },
    Tuple(Vec<TyScrutinee>),
}

#[derive(Debug, Clone)]
pub(crate) struct TyStructScrutineeField {
    pub(crate) field: Ident,
    pub(crate) scrutinee: Option<TyScrutinee>,
    pub(crate) span: Span,
}
