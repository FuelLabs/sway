use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::DeclRefMixedInterface, engine_threading::*, has_changes, language::CallPath,
    type_system::*,
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

impl TyImplTrait {
    pub fn is_impl_contract(&self, te: &TypeEngine) -> bool {
        matches!(&*te.get(self.implementing_for.type_id), TypeInfo::Contract)
    }
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
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.impl_type_parameters
            .eq(&other.impl_type_parameters, ctx)
            && self.trait_name == other.trait_name
            && self
                .trait_type_arguments
                .eq(&other.trait_type_arguments, ctx)
            && self.items.eq(&other.items, ctx)
            && self.implementing_for.eq(&other.implementing_for, ctx)
            && self.trait_decl_ref.eq(&other.trait_decl_ref, ctx)
    }
}

impl HashWithEngines for TyImplTrait {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
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
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) -> HasChanges {
        has_changes! {
            self.impl_type_parameters.subst(type_mapping, engines);
            self.implementing_for.subst_inner(type_mapping, engines);
            self.items.subst(type_mapping, engines);
        }
    }
}
