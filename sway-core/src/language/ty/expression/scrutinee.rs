use crate::{
    decl_engine::{DeclRefEnum, DeclRefStruct},
    language::{ty::*, *},
    type_system::*,
};
use serde::{Deserialize, Serialize};
use sway_types::{Ident, Span};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TyScrutinee {
    pub variant: TyScrutineeVariant,
    pub type_id: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum TyScrutineeVariant {
    Or(Vec<TyScrutinee>),
    CatchAll,
    Literal(Literal),
    Variable(Ident),
    Constant(Ident, Literal, TyConstantDecl),
    StructScrutinee {
        struct_ref: DeclRefStruct,
        fields: Vec<TyStructScrutineeField>,
        instantiation_call_path: CallPath,
    },
    EnumScrutinee {
        enum_ref: DeclRefEnum,
        variant: Box<TyEnumVariant>,
        /// Should contain a TyDecl to either an enum or a type alias.
        call_path_decl: ty::TyDecl,
        value: Box<TyScrutinee>,
        instantiation_call_path: CallPath,
    },
    Tuple(Vec<TyScrutinee>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TyStructScrutineeField {
    pub field: Ident,
    pub scrutinee: Option<TyScrutinee>,
    pub span: Span,
    pub field_def_name: Ident,
}
