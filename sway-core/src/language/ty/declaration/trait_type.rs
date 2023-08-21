use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{engine_threading::*, transform, type_system::*};

#[derive(Clone, Debug)]
pub struct TyTraitType {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub ty: Option<TypeArgument>,
    pub span: Span,
}

impl Named for TyTraitType {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl EqWithEngines for TyTraitType {}
impl PartialEqWithEngines for TyTraitType {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.name == other.name && self.ty.eq(&other.ty, engines)
    }
}

impl HashWithEngines for TyTraitType {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyTraitType {
            name,
            ty,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        name.hash(state);
        ty.hash(state, engines);
    }
}

impl SubstTypes for TyTraitType {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        if let Some(ref mut ty) = self.ty {
            ty.subst(type_mapping, engines);
        }
    }
}

impl ReplaceSelfType for TyTraitType {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        if let Some(ref mut ty) = self.ty {
            ty.replace_self_type(engines, self_type);
        }
    }
}

impl Spanned for TyTraitType {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
