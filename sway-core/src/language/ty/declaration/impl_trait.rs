use std::hash::{Hash, Hasher};

use sway_types::Span;

use crate::{decl_engine::DeclRef, engine_threading::*, language::CallPath, type_system::*};

#[derive(Clone, Debug)]
pub struct TyImplTrait {
    pub impl_type_parameters: Vec<TypeParameter>,
    pub trait_name: CallPath,
    pub trait_type_arguments: Vec<TypeArgument>,
    pub methods: Vec<DeclRef>,
    pub implementing_for_type_id: TypeId,
    pub trait_decl_ref: Option<DeclRef>,
    pub type_implementing_for_span: Span,
    pub span: Span,
}

impl EqWithEngines for TyImplTrait {}
impl PartialEqWithEngines for TyImplTrait {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        self.impl_type_parameters
            .eq(&other.impl_type_parameters, engines)
            && self.trait_name == other.trait_name
            && self
                .trait_type_arguments
                .eq(&other.trait_type_arguments, engines)
            && self.methods.eq(&other.methods, engines)
            && type_engine
                .get(self.implementing_for_type_id)
                .eq(&type_engine.get(other.implementing_for_type_id), engines)
            && self.trait_decl_ref.eq(&other.trait_decl_ref, engines)
    }
}

impl HashWithEngines for TyImplTrait {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TyImplTrait {
            impl_type_parameters,
            trait_name,
            trait_type_arguments,
            methods,
            implementing_for_type_id,
            trait_decl_ref,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            type_implementing_for_span: _,
            span: _,
        } = self;
        let type_engine = engines.te();
        trait_name.hash(state);
        impl_type_parameters.hash(state, engines);
        trait_type_arguments.hash(state, engines);
        methods.hash(state, engines);
        type_engine
            .get(*implementing_for_type_id)
            .hash(state, engines);
        trait_decl_ref.hash(state, engines);
    }
}

impl SubstTypes for TyImplTrait {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
        self.implementing_for_type_id.subst(type_mapping, engines);
        self.methods
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl ReplaceSelfType for TyImplTrait {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.impl_type_parameters
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
        self.implementing_for_type_id
            .replace_self_type(engines, self_type);
        self.methods
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}
