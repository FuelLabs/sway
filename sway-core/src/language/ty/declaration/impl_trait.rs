use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::DeclRefMixedInterface, engine_threading::*, language::CallPath, type_system::*,
};

use super::TyTraitItem;

pub type TyImplItem = TyTraitItem;

// impl <A, B, C> Trait<Arg, Arg> for Type<Arg, Arg>
#[derive(Clone, Debug)]
pub struct TyImplTrait {
    pub impl_type_parameters: Vec<TypeParameter>,
    pub trait_name: CallPath,
    pub trait_type_arguments: Vec<TypeArgument>,
    pub items: Vec<TyImplItem>,
    pub trait_decl_ref: Option<DeclRefMixedInterface>,
    pub implementing_for: TypeArgument,
    pub span: Span,
}

impl Named for TyImplTrait {
    fn name(&self) -> &Ident {
        &self.trait_name.suffix
    }
}

impl Spanned for TyImplTrait {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl EqWithEngines for TyImplTrait {}
impl PartialEqWithEngines for TyImplTrait {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.impl_type_parameters
            .eq(&other.impl_type_parameters, engines)
            && self.trait_name == other.trait_name
            && self
                .trait_type_arguments
                .eq(&other.trait_type_arguments, engines)
            && self.items.eq(&other.items, engines)
            && self.implementing_for.eq(&other.implementing_for, engines)
            && self.trait_decl_ref.eq(&other.trait_decl_ref, engines)
    }
}

impl HashWithEngines for TyImplTrait {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyImplTrait {
            impl_type_parameters,
            trait_name,
            trait_type_arguments,
            items,
            implementing_for,
            trait_decl_ref,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
        } = self;
        trait_name.hash(state);
        impl_type_parameters.hash(state, engines);
        trait_type_arguments.hash(state, engines);
        items.hash(state, engines);
        implementing_for.hash(state, engines);
        trait_decl_ref.hash(state, engines);
    }
}

impl SubstTypes for TyImplTrait {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.implementing_for.subst_inner(type_mapping, engines);
        self.items
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl ReplaceSelfType for TyImplTrait {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.implementing_for.replace_self_type(engines, self_type);
        self.items
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}
