use std::borrow::Cow;

use sway_types::{state::StateIndex, Ident, Span, Spanned};

use crate::{
    declaration_engine::{DeclMapping, ReplaceDecls},
    engine_threading::*,
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

impl EqWithEngines for TyReassignment {}
impl PartialEqWithEngines for TyReassignment {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.lhs_base_name == other.lhs_base_name
            && self.lhs_type == other.lhs_type
            && self.lhs_indices.eq(&other.lhs_indices, engines)
            && self.rhs.eq(&other.rhs, engines)
    }
}

impl CopyTypes for TyReassignment {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        self.rhs.copy_types(type_mapping, engines);
        self.lhs_type.copy_types(type_mapping, engines);
    }
}

impl ReplaceSelfType for TyReassignment {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.rhs.replace_self_type(engines, self_type);
        self.lhs_type.replace_self_type(engines, self_type);
    }
}

impl ReplaceDecls for TyReassignment {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        self.rhs.replace_decls(decl_mapping, engines);
    }
}

#[derive(Clone, Debug)]
pub enum ProjectionKind {
    StructField {
        name: Ident,
    },
    TupleField {
        index: usize,
        index_span: Span,
    },
    ArrayIndex {
        index: Box<TyExpression>,
        index_span: Span,
    },
}

impl EqWithEngines for ProjectionKind {}
impl PartialEqWithEngines for ProjectionKind {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        match (self, other) {
            (
                ProjectionKind::StructField { name: l_name },
                ProjectionKind::StructField { name: r_name },
            ) => l_name == r_name,
            (
                ProjectionKind::TupleField {
                    index: l_index,
                    index_span: l_index_span,
                },
                ProjectionKind::TupleField {
                    index: r_index,
                    index_span: r_index_span,
                },
            ) => l_index == r_index && l_index_span == r_index_span,
            (
                ProjectionKind::ArrayIndex {
                    index: l_index,
                    index_span: l_index_span,
                },
                ProjectionKind::ArrayIndex {
                    index: r_index,
                    index_span: r_index_span,
                },
            ) => l_index.eq(r_index, engines) && l_index_span == r_index_span,
            _ => false,
        }
    }
}

impl Spanned for ProjectionKind {
    fn span(&self) -> Span {
        match self {
            ProjectionKind::StructField { name } => name.span(),
            ProjectionKind::TupleField { index_span, .. } => index_span.clone(),
            ProjectionKind::ArrayIndex { index_span, .. } => index_span.clone(),
        }
    }
}

impl ProjectionKind {
    pub(crate) fn pretty_print(&self) -> Cow<str> {
        match self {
            ProjectionKind::StructField { name } => Cow::Borrowed(name.as_str()),
            ProjectionKind::TupleField { index, .. } => Cow::Owned(index.to_string()),
            ProjectionKind::ArrayIndex { index, .. } => Cow::Owned(format!("{:#?}", index)),
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

impl EqWithEngines for TyStorageReassignment {}
impl PartialEqWithEngines for TyStorageReassignment {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.fields.eq(&other.fields, engines)
            && self.ix == other.ix
            && self.rhs.eq(&other.rhs, engines)
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
impl EqWithEngines for TyStorageReassignDescriptor {}
impl PartialEqWithEngines for TyStorageReassignDescriptor {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.name == other.name
            && type_engine
                .look_up_type_id(self.type_id)
                .eq(&type_engine.look_up_type_id(other.type_id), engines)
    }
}
