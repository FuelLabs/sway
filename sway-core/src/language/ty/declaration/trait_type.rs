use std::{
    fmt,
    hash::{Hash, Hasher},
};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{engine_threading::*, has_changes, transform, type_system::*};

#[derive(Clone, Debug)]
pub struct TyTraitType {
    pub name: Ident,
    pub attributes: transform::AttributesMap,
    pub ty: Option<TypeArgument>,
    pub implementing_type: TypeId,
    pub span: Span,
}

impl DebugWithEngines for TyTraitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, _engines: &Engines) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Named for TyTraitType {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl EqWithEngines for TyTraitType {}
impl PartialEqWithEngines for TyTraitType {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name
            && self.ty.eq(&other.ty, ctx)
            && self.implementing_type.eq(&other.implementing_type)
    }
}

impl HashWithEngines for TyTraitType {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyTraitType {
            name,
            ty,
            implementing_type,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            attributes: _,
        } = self;
        name.hash(state);
        ty.hash(state, engines);
        implementing_type.hash(state);
    }
}

impl SubstTypes for TyTraitType {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        has_changes! {
            self.ty.subst(type_mapping, engines);
            self.implementing_type.subst(type_mapping, engines);
        }
    }
}

impl Spanned for TyTraitType {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
