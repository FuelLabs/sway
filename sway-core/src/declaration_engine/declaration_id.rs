use sway_types::{Span, Spanned};

use crate::{
    engine_threading::*,
    language::ty,
    type_system::{CopyTypes, TypeMapping},
    ReplaceSelfType, TypeId,
};

use super::{DeclMapping, DeclarationEngine, ReplaceDecls, ReplaceFunctionImplementingType};

/// An ID used to refer to an item in the [DeclarationEngine](super::declaration_engine::DeclarationEngine)
#[derive(Debug)]
pub struct DeclarationId(usize, Span);

impl Clone for DeclarationId {
    fn clone(&self) -> Self {
        Self(self.0, self.1.clone())
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithEngines for DeclarationId {}
impl PartialEqWithEngines for DeclarationId {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let declaration_engine = engines.de();
        let left = declaration_engine.look_up_decl_id(self.clone());
        let right = declaration_engine.look_up_decl_id(other.clone());
        left.eq(&right, engines)
    }
}

impl std::ops::Deref for DeclarationId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(clippy::from_over_into)]
impl Into<usize> for DeclarationId {
    fn into(self) -> usize {
        self.0
    }
}

impl Spanned for DeclarationId {
    fn span(&self) -> Span {
        self.1.clone()
    }
}

impl CopyTypes for DeclarationId {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        let declaration_engine = engines.de();
        let mut decl = declaration_engine.look_up_decl_id(self.clone());
        decl.copy_types(type_mapping, engines);
        declaration_engine.replace_decl_id(self.clone(), decl);
    }
}

impl ReplaceSelfType for DeclarationId {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let declaration_engine = engines.de();
        let mut decl = declaration_engine.look_up_decl_id(self.clone());
        decl.replace_self_type(engines, self_type);
        declaration_engine.replace_decl_id(self.clone(), decl);
    }
}

impl ReplaceDecls for DeclarationId {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        let declaration_engine = engines.de();
        if let Some(new_decl_id) = decl_mapping.find_match(self) {
            self.0 = *new_decl_id;
            return;
        }
        let all_parents = declaration_engine.find_all_parents(engines, self.clone());
        for parent in all_parents.into_iter() {
            if let Some(new_decl_id) = decl_mapping.find_match(&parent) {
                self.0 = *new_decl_id;
                return;
            }
        }
    }
}

impl ReplaceFunctionImplementingType for DeclarationId {
    fn replace_implementing_type(
        &mut self,
        engines: Engines<'_>,
        implementing_type: ty::TyDeclaration,
    ) {
        let declaration_engine = engines.de();
        let mut decl = declaration_engine.look_up_decl_id(self.clone());
        decl.replace_implementing_type(engines, implementing_type);
        declaration_engine.replace_decl_id(self.clone(), decl);
    }
}

impl DeclarationId {
    pub(crate) fn new(index: usize, span: Span) -> DeclarationId {
        DeclarationId(index, span)
    }

    pub(crate) fn with_parent(
        self,
        declaration_engine: &DeclarationEngine,
        parent: DeclarationId,
    ) -> DeclarationId {
        declaration_engine.register_parent(&self, parent);
        self
    }

    pub(crate) fn replace_id(&mut self, index: usize) {
        self.0 = index;
    }

    pub(crate) fn copy_types_and_insert_new(
        &self,
        type_mapping: &TypeMapping,
        engines: Engines<'_>,
    ) -> DeclarationId {
        let declaration_engine = engines.de();
        let mut decl = declaration_engine.look_up_decl_id(self.clone());
        decl.copy_types(type_mapping, engines);
        declaration_engine
            .insert(decl, self.1.clone())
            .with_parent(declaration_engine, self.clone())
    }

    pub(crate) fn replace_self_type_and_insert_new(
        &self,
        engines: Engines<'_>,
        self_type: TypeId,
    ) -> DeclarationId {
        let declaration_engine = engines.de();
        let mut decl = declaration_engine.look_up_decl_id(self.clone());
        decl.replace_self_type(engines, self_type);
        declaration_engine
            .insert(decl, self.1.clone())
            .with_parent(declaration_engine, self.clone())
    }

    pub(crate) fn replace_decls_and_insert_new(
        &self,
        decl_mapping: &DeclMapping,
        engines: Engines<'_>,
    ) -> DeclarationId {
        let declaration_engine = engines.de();
        let mut decl = declaration_engine.look_up_decl_id(self.clone());
        decl.replace_decls(decl_mapping, engines);
        declaration_engine
            .insert(decl, self.1.clone())
            .with_parent(declaration_engine, self.clone())
    }
}
