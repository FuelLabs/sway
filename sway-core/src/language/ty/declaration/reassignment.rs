use sway_types::Ident;

use crate::{language::ty::*, semantic_analysis::ProjectionKind, type_system::*};

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
