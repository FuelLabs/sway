use crate::{
    decl_engine::DeclRef, engine_threading::*, language::parsed, transform, type_system::*,
};
use std::hash::{Hash, Hasher};

use sway_types::{Ident, Span, Spanned};

/// A [TyAbiDeclaration] contains the type-checked version of the parse tree's `AbiDeclaration`.
#[derive(Clone, Debug)]
pub struct TyAbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    pub implementing_for: TypeParameter,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<DeclRef>,
    pub supertraits: Vec<parsed::Supertrait>,
    pub methods: Vec<DeclRef>,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl EqWithEngines for TyAbiDeclaration {}
impl PartialEqWithEngines for TyAbiDeclaration {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.implementing_for.eq(&other.implementing_for, engines)
            && self.interface_surface.eq(&other.interface_surface, engines)
            && self.methods.eq(&other.methods, engines)
    }
}

impl HashWithEngines for TyAbiDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyAbiDeclaration {
            name,
            implementing_for,
            interface_surface,
            methods,
            supertraits,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            attributes: _,
            span: _,
        } = self;
        name.hash(state);
        implementing_for.hash(state, engines);
        interface_surface.hash(state, engines);
        methods.hash(state, engines);
        supertraits.hash(state, engines);
    }
}

impl CreateTypeId for TyAbiDeclaration {
    fn create_type_id(&self, engines: Engines<'_>) -> TypeId {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let ty = TypeInfo::ContractCaller {
            abi_name: AbiName::Known(self.name.clone().into()),
            address: None,
        };
        type_engine.insert(decl_engine, ty)
    }
}

impl Spanned for TyAbiDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl SubstTypes for TyAbiDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let TyAbiDeclaration {
            implementing_for,
            interface_surface,
            methods,
            // these fields are not used because they don't contain types
            name: _,
            supertraits: _,
            attributes: _,
            span: _,
        } = self;
        implementing_for.subst(type_mapping, engines);
        interface_surface.iter_mut().for_each(|decl_ref| {
            let new_decl_ref = decl_ref
                .clone()
                .subst_types_and_insert_new(type_mapping, engines);
            decl_ref.replace_id((&new_decl_ref).into());
        });
        methods.iter_mut().for_each(|decl_ref| {
            let new_decl_ref = decl_ref
                .clone()
                .subst_types_and_insert_new(type_mapping, engines);
            decl_ref.replace_id((&new_decl_ref).into());
        });
    }
}
