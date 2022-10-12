use std::borrow::Cow;

use sway_types::{Ident, Span, Spanned};

use crate::{language::ty::*, type_system::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyReassignment {
    // either a direct variable, so length of 1, or
    // at series of struct fields/array indices (array syntax)
    pub lhs_base_name: Ident,
    pub lhs_type: TypeId,
    pub lhs_indices: Vec<ProjectionKind>,
    pub rhs: TyExpression,
}

impl CopyTypes for TyReassignment {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.rhs.copy_types(type_mapping);
        self.lhs_type.copy_types(type_mapping);
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
