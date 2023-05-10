use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

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

impl Named for TyTraitFn {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Spanned for TyTraitFn {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl EqWithEngines for TyTraitFn {}
impl PartialEqWithEngines for TyTraitFn {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.name == other.name
            && self.purity == other.purity
            && self.parameters.eq(&other.parameters, engines)
            && type_engine
                .get(self.return_type)
                .eq(&type_engine.get(other.return_type), engines)
            && self.attributes == other.attributes
    }
}

impl HashWithEngines for TyTraitFn {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyTraitFn {
            name,
            purity,
            parameters,
            return_type,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            return_type_span: _,
            attributes: _,
        } = self;
        let type_engine = engines.te();
        name.hash(state);
        parameters.hash(state, engines);
        type_engine.get(*return_type).hash(state, engines);
        purity.hash(state);
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
