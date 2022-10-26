use derivative::Derivative;
use sway_types::{Ident, Span};

use crate::{
    language::{ty::*, Purity},
    transform,
    type_system::*,
};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TyTraitFn {
    pub name: Ident,
    pub(crate) purity: Purity,
    pub parameters: Vec<TyFunctionParameter>,
    pub return_type: TypeId,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub return_type_span: Span,
    pub attributes: transform::AttributesMap,
}

impl CopyTypes for TyTraitFn {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        self.parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.return_type.copy_types(type_mapping);
    }
}

impl ReplaceSelfType for TyTraitFn {
    fn replace_self_type(&mut self, self_type: TypeId) {
        self.parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(self_type));
        self.return_type.replace_self_type(self_type);
    }
}

impl MonomorphizeHelper for TyTraitFn {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn type_parameters(&self) -> &[TypeParameter] {
        &[]
    }
}
