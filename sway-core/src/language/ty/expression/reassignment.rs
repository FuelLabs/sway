use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};

use sway_types::{state::StateIndex, Ident, Span, Spanned};

use crate::{engine_threading::*, language::ty::*, type_system::*};

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
        let type_engine = engines.te();
        self.lhs_base_name == other.lhs_base_name
            && type_engine
                .get(self.lhs_type)
                .eq(&type_engine.get(other.lhs_type), engines)
            && self.lhs_indices.eq(&other.lhs_indices, engines)
            && self.rhs.eq(&other.rhs, engines)
    }
}

impl HashWithEngines for TyReassignment {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyReassignment {
            lhs_base_name,
            lhs_type,
            lhs_indices,
            rhs,
        } = self;
        let type_engine = engines.te();
        lhs_base_name.hash(state);
        type_engine.get(*lhs_type).hash(state, engines);
        lhs_indices.hash(state, engines);
        rhs.hash(state, engines);
    }
}

impl SubstTypes for TyReassignment {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.rhs.subst(type_mapping, engines);
        self.lhs_type.subst(type_mapping, engines);
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

impl HashWithEngines for ProjectionKind {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        use ProjectionKind::*;
        std::mem::discriminant(self).hash(state);
        match self {
            StructField { name } => name.hash(state),
            TupleField {
                index,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                index_span: _,
            } => index.hash(state),
            ArrayIndex {
                index,
                // these fields are not hashed because they aren't relevant/a
                // reliable source of obj v. obj distinction
                index_span: _,
            } => {
                index.hash(state, engines);
            }
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
            ProjectionKind::ArrayIndex { index, .. } => Cow::Owned(format!("{index:#?}")),
        }
    }
}

/// Describes each field being drilled down into in storage and its type.
#[derive(Clone, Debug)]
pub struct TyStorageReassignment {
    pub fields: Vec<TyStorageReassignDescriptor>,
    pub(crate) ix: StateIndex,
    pub rhs: TyExpression,
    pub storage_keyword_span: Span,
}

impl EqWithEngines for TyStorageReassignment {}
impl PartialEqWithEngines for TyStorageReassignment {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.fields.eq(&other.fields, engines)
            && self.ix == other.ix
            && self.rhs.eq(&other.rhs, engines)
    }
}

impl HashWithEngines for TyStorageReassignment {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyStorageReassignment {
            fields,
            ix,
            rhs,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            storage_keyword_span: _,
        } = self;
        fields.hash(state, engines);
        ix.hash(state);
        rhs.hash(state, engines);
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

impl EqWithEngines for TyStorageReassignDescriptor {}
impl PartialEqWithEngines for TyStorageReassignDescriptor {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.name == other.name
            && type_engine
                .get(self.type_id)
                .eq(&type_engine.get(other.type_id), engines)
    }
}

impl HashWithEngines for TyStorageReassignDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyStorageReassignDescriptor {
            name,
            type_id,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
        } = self;
        let type_engine = engines.te();
        name.hash(state);
        type_engine.get(*type_id).hash(state, engines);
    }
}
