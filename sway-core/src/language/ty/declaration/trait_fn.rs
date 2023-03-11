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

impl SubstTypes for TyTraitFn {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.return_type.subst(type_mapping, engines);
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
