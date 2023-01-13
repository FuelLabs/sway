use sway_types::{Ident, Span};

use crate::{
    engine_threading::*,
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

impl EqWithEngines for TyTraitFn {}
impl PartialEqWithEngines for TyTraitFn {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.purity == other.purity
            && self.parameters.eq(&other.parameters, engines)
            && self.return_type == other.return_type
            && self.attributes == other.attributes
    }
}

impl SubstTypes for TyTraitFn {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.return_type.subst(type_mapping, engines);
    }
}

impl ReplaceSelfType for TyTraitFn {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.return_type.replace_self_type(engines, self_type);
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
