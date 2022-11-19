use sway_types::{Ident, Span};

use crate::{
    language::{ty::*, Purity},
    transform,
    type_system::*,
};

#[derive(Clone, Debug)]
pub struct TyTraitFn {
    pub name: Ident,
    pub(crate) purity: Purity,
    pub parameters: Vec<TyFunctionParameter>,
    pub return_type: TypeId,
    pub return_type_span: Span,
    pub attributes: transform::AttributesMap,
}

impl EqWithTypeEngine for TyTraitFn {}
impl PartialEqWithTypeEngine for TyTraitFn {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.name == rhs.name
            && self.purity == rhs.purity
            && self.parameters.eq(&rhs.parameters, type_engine)
            && self.return_type == rhs.return_type
            && self.attributes == rhs.attributes
    }
}

impl CopyTypes for TyTraitFn {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        self.parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping, type_engine));
        self.return_type.copy_types(type_mapping, type_engine);
    }
}

impl ReplaceSelfType for TyTraitFn {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        self.parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(type_engine, self_type));
        self.return_type.replace_self_type(type_engine, self_type);
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
