use sway_types::{Ident, Span};

use crate::{
    language::{ty::*, *},
    type_system::*,
};

#[derive(Debug, Clone)]
pub struct TyScrutinee {
    pub variant: TyScrutineeVariant,
    pub type_id: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TyScrutineeVariant {
    CatchAll,
    Literal(Literal),
    Variable(Ident),
    Constant(Ident, Literal, TyConstantDeclaration),
    StructScrutinee {
        struct_name: Ident,
        decl_name: Ident,
        fields: Vec<TyStructScrutineeField>,
    },
    #[allow(dead_code)]
    EnumScrutinee {
        call_path: CallPath,
        variant: Box<TyEnumVariant>,
        decl_name: Ident,
        value: Box<TyScrutinee>,
    },
    Tuple(Vec<TyScrutinee>),
}

#[derive(Debug, Clone)]
pub struct TyStructScrutineeField {
    pub field: Ident,
    pub scrutinee: Option<TyScrutinee>,
    pub span: Span,
    pub field_def_name: Ident,
}
