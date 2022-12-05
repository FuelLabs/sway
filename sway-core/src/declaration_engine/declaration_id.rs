use std::fmt;

use sway_types::{Span, Spanned};

use crate::{
    engine_threading::*,
    type_system::{CopyTypes, TypeMapping},
    ReplaceSelfType, TypeEngine, TypeId,
};

use super::{
    de_find_all_parents,
    declaration_engine::{de_look_up_decl_id, de_replace_decl_id},
    DeclMapping, DeclarationEngine, ReplaceDecls,
};

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
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        de_look_up_decl_id(self.clone()).eq(&de_look_up_decl_id(other.clone()), type_engine)
    }
}

impl fmt::Display for DeclarationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&de_look_up_decl_id(self.clone()).to_string())
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
        let mut decl = de_look_up_decl_id(self.clone());
        decl.copy_types(type_mapping, engines);
        de_replace_decl_id(self.clone(), decl);
    }
}

impl ReplaceSelfType for DeclarationId {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.replace_self_type(engines, self_type);
        de_replace_decl_id(self.clone(), decl);
    }
}

impl ReplaceDecls for DeclarationId {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        if let Some(new_decl_id) = decl_mapping.find_match(self) {
            self.0 = *new_decl_id;
            return;
        }
        let all_parents = de_find_all_parents(self.clone(), engines.te());
        for parent in all_parents.into_iter() {
            if let Some(new_decl_id) = decl_mapping.find_match(&parent) {
                self.0 = *new_decl_id;
                return;
            }
        }
    }
}

impl DeclarationId {
    pub(super) fn new(index: usize, span: Span) -> DeclarationId {
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
        let mut decl = de_look_up_decl_id(self.clone());
        decl.copy_types(type_mapping, engines);
        engines
            .de()
            .insert(decl, self.1.clone())
            .with_parent(engines.de(), self.clone())
    }

    pub(crate) fn replace_self_type_and_insert_new(
        &self,
        engines: Engines<'_>,
        self_type: TypeId,
    ) -> DeclarationId {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.replace_self_type(engines, self_type);
        engines
            .de()
            .insert(decl, self.1.clone())
            .with_parent(engines.de(), self.clone())
    }

    pub(crate) fn replace_decls_and_insert_new(
        &self,
        decl_mapping: &DeclMapping,
        engines: Engines<'_>,
    ) -> DeclarationId {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.replace_decls(decl_mapping, engines);
        engines
            .de()
            .insert(decl, self.1.clone())
            .with_parent(engines.de(), self.clone())
    }
}
