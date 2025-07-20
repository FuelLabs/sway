use super::{TyAbiDecl, TyDeclParsedType, TyTraitDecl, TyTraitItem};
use crate::{
    decl_engine::{DeclId, DeclRefMixedInterface, InterfaceDeclId},
    engine_threading::*,
    has_changes,
    language::{parsed::ImplSelfOrTrait, CallPath},
    type_system::*,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Formatter,
    hash::{Hash, Hasher},
};
use sway_types::{Ident, Named, Span, Spanned};

pub type TyImplItem = TyTraitItem;

// impl <A, B, C> Trait<Arg, Arg> for Type<Arg, Arg>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TyImplSelfOrTrait {
    pub impl_type_parameters: Vec<TypeParameter>,
    pub trait_name: CallPath,
    pub trait_type_arguments: Vec<GenericArgument>,
    pub items: Vec<TyImplItem>,
    pub supertrait_items: Vec<TyImplItem>,
    pub trait_decl_ref: Option<DeclRefMixedInterface>,
    pub implementing_for: GenericArgument,
    pub span: Span,
}

impl TyImplSelfOrTrait {
    pub fn is_impl_contract(&self, te: &TypeEngine) -> bool {
        matches!(
            &*te.get(self.implementing_for.type_id()),
            TypeInfo::Contract
        )
    }

    pub fn is_impl_self(&self) -> bool {
        self.trait_decl_ref.is_none()
    }

    pub fn is_impl_trait(&self) -> bool {
        match &self.trait_decl_ref {
            Some(decl_ref) => matches!(decl_ref.id(), InterfaceDeclId::Trait(_)),
            _ => false,
        }
    }

    pub fn is_impl_abi(&self) -> bool {
        match &self.trait_decl_ref {
            Some(decl_ref) => matches!(decl_ref.id(), InterfaceDeclId::Abi(_)),
            _ => false,
        }
    }

    /// Returns [DeclId] of the trait implemented by `self`, if `self` implements a trait.
    pub fn implemented_trait_decl_id(&self) -> Option<DeclId<TyTraitDecl>> {
        match &self.trait_decl_ref {
            Some(decl_ref) => match &decl_ref.id() {
                InterfaceDeclId::Trait(decl_id) => Some(*decl_id),
                InterfaceDeclId::Abi(_) => None,
            },
            _ => None,
        }
    }

    /// Returns [DeclId] of the ABI implemented by `self`, if `self` implements an ABI for a contract.
    pub fn implemented_abi_decl_id(&self) -> Option<DeclId<TyAbiDecl>> {
        match &self.trait_decl_ref {
            Some(decl_ref) => match &decl_ref.id() {
                InterfaceDeclId::Abi(decl_id) => Some(*decl_id),
                InterfaceDeclId::Trait(_) => None,
            },
            _ => None,
        }
    }
}

impl TyDeclParsedType for TyImplSelfOrTrait {
    type ParsedType = ImplSelfOrTrait;
}

impl Named for TyImplSelfOrTrait {
    fn name(&self) -> &Ident {
        &self.trait_name.suffix
    }
}

impl Spanned for TyImplSelfOrTrait {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl EqWithEngines for TyImplSelfOrTrait {}
impl PartialEqWithEngines for TyImplSelfOrTrait {
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

impl HashWithEngines for TyImplSelfOrTrait {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TyImplSelfOrTrait {
            impl_type_parameters,
            trait_name,
            trait_type_arguments,
            items,
            implementing_for,
            trait_decl_ref,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            span: _,
            supertrait_items: _,
        } = self;
        trait_name.hash(state);
        impl_type_parameters.hash(state, engines);
        trait_type_arguments.hash(state, engines);
        items.hash(state, engines);
        implementing_for.hash(state, engines);
        trait_decl_ref.hash(state, engines);
    }
}

impl SubstTypes for TyImplSelfOrTrait {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        has_changes! {
            self.impl_type_parameters.subst(ctx);
            self.implementing_for.subst_inner(ctx);
            self.items.subst(ctx);
        }
    }
}

impl DebugWithEngines for TyImplSelfOrTrait {
    fn fmt(&self, f: &mut Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        if let Some(t) = self.trait_decl_ref.as_ref() {
            write!(
                f,
                "impl<> {:?} for {:?} -> {:?}",
                t.name().as_str(),
                engines.help_out(self.implementing_for.initial_type_id()),
                engines.help_out(self.implementing_for.type_id()),
            )
        } else {
            write!(
                f,
                "impl<> {:?} -> {:?}",
                engines.help_out(self.implementing_for.initial_type_id()),
                engines.help_out(self.implementing_for.type_id()),
            )
        }
    }
}
