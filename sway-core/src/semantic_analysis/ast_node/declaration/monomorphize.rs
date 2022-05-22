use sway_types::{Ident, Span};

use crate::{namespace::Items, semantic_analysis::TypeMapping, TypeParameter};

pub(crate) trait MonomorphizeHelper {
    type Output;

    fn type_parameters(&self) -> &[TypeParameter];
    fn name(&self) -> &Ident;
    fn span(&self) -> &Span;
    fn monomorphize_inner(self, type_mapping: &TypeMapping, namespace: &mut Items) -> Self::Output;
}
