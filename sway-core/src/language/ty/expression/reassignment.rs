use std::borrow::Cow;

use sway_types::{state::StateIndex, Ident, Span, Spanned};

use crate::{
    declaration_engine::{DeclMapping, ReplaceDecls},
    language::ty::*,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyReassignment {
    // either a direct variable, so length of 1, or
    // at series of struct fields/array indices (array syntax)
    pub lhs_base_name: Ident,
    pub lhs_type: TypeId,
    pub lhs_indices: Vec<ProjectionKind>,
    pub rhs: TyExpression,
}

impl EqWithTypeEngine for TyReassignment {}
impl PartialEqWithTypeEngine for TyReassignment {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.lhs_base_name == rhs.lhs_base_name
            && self.lhs_type == rhs.lhs_type
            && self.lhs_indices == rhs.lhs_indices
            && self.rhs.eq(&rhs.rhs, type_engine)
    }
}

impl CopyTypes for TyReassignment {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        self.rhs.copy_types(type_mapping, type_engine);
        self.lhs_type.copy_types(type_mapping, type_engine);
    }
}

impl ReplaceSelfType for TyReassignment {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        self.rhs.replace_self_type(type_engine, self_type);
        self.lhs_type.replace_self_type(type_engine, self_type);
    }
}

impl ReplaceDecls for TyReassignment {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, type_engine: &TypeEngine) {
        self.rhs.replace_decls(decl_mapping, type_engine);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionKind {
    StructField { name: Ident },
    TupleField { index: usize, index_span: Span },
}

impl Spanned for ProjectionKind {
    fn span(&self) -> Span {
        match self {
            ProjectionKind::StructField { name } => name.span(),
            ProjectionKind::TupleField { index_span, .. } => index_span.clone(),
        }
    }
}

impl ProjectionKind {
    pub(crate) fn pretty_print(&self) -> Cow<str> {
        match self {
            ProjectionKind::StructField { name } => Cow::Borrowed(name.as_str()),
            ProjectionKind::TupleField { index, .. } => Cow::Owned(index.to_string()),
        }
    }
}

/// Describes each field being drilled down into in storage and its type.
#[derive(Clone, Debug)]
pub struct TyStorageReassignment {
    pub fields: Vec<TyStorageReassignDescriptor>,
    pub(crate) ix: StateIndex,
    pub rhs: TyExpression,
}

impl EqWithTypeEngine for TyStorageReassignment {}
impl PartialEqWithTypeEngine for TyStorageReassignment {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.fields.eq(&rhs.fields, type_engine)
            && self.ix == rhs.ix
            && self.rhs.eq(&rhs.rhs, type_engine)
    }
}

impl Spanned for TyStorageReassignment {
    fn span(&self) -> Span {
        self.fields
            .iter()
            .fold(self.fields[0].span.clone(), |acc, field| {
                Span::join(acc, field.span.clone())
            })
    }
}

impl TyStorageReassignment {
    pub fn names(&self) -> Vec<Ident> {
        self.fields
            .iter()
            .map(|f| f.name.clone())
            .collect::<Vec<_>>()
    }
}

/// Describes a single subfield access in the sequence when reassigning to a subfield within
/// storage.
#[derive(Clone, Debug)]
pub struct TyStorageReassignDescriptor {
    pub name: Ident,
    pub type_id: TypeId,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithTypeEngine for TyStorageReassignDescriptor {}
impl PartialEqWithTypeEngine for TyStorageReassignDescriptor {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.name == other.name
            && type_engine
                .look_up_type_id(self.type_id)
                .eq(&type_engine.look_up_type_id(other.type_id), type_engine)
    }
}
