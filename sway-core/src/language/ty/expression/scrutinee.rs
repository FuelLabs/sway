use sway_types::{Ident, Span};

use crate::{
    decl_engine::{DeclRefEnum, DeclRefStruct},
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
    Or(Vec<TyScrutinee>),
    CatchAll,
    Literal(Literal),
    Variable(Ident),
    Constant(Ident, Literal, TyConstantDecl),
    StructScrutinee {
        struct_ref: DeclRefStruct,
        fields: Vec<TyStructScrutineeField>,
        instantiation_symbol_path: SymbolPath,
    },
    EnumScrutinee {
        enum_ref: DeclRefEnum,
        variant: Box<TyEnumVariant>,
        /// Should contain a TyDecl to either an enum or a type alias.
        symbol_path_decl: ty::TyDecl,
        value: Box<TyScrutinee>,
        instantiation_symbol_path: SymbolPath,
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
