use std::fmt;

use sway_types::{Span, Spanned};

use crate::{
    language::ty::TyDeclaration,
    type_system::{CopyTypes, TypeMapping},
    EqWithTypeEngine, PartialEqWithTypeEngine, ReplaceSelfType, TypeEngine, TypeId,
};

use super::{
    de_find_all_parents, de_insert, de_register_parent,
    declaration_engine::{de_look_up_decl_id, de_replace_decl_id},
    DeclMapping, ReplaceDecls, ReplaceFunctionImplementingType,
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
impl EqWithTypeEngine for DeclarationId {}
impl PartialEqWithTypeEngine for DeclarationId {
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
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.copy_types(type_mapping, type_engine);
        de_replace_decl_id(self.clone(), decl);
    }
}

impl ReplaceSelfType for DeclarationId {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.replace_self_type(type_engine, self_type);
        de_replace_decl_id(self.clone(), decl);
    }
}

impl ReplaceDecls for DeclarationId {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, type_engine: &TypeEngine) {
        if let Some(new_decl_id) = decl_mapping.find_match(self) {
            self.0 = *new_decl_id;
            return;
        }
        let all_parents = de_find_all_parents(self.clone(), type_engine);
        for parent in all_parents.into_iter() {
            if let Some(new_decl_id) = decl_mapping.find_match(&parent) {
                self.0 = *new_decl_id;
                return;
            }
        }
    }
}

impl ReplaceFunctionImplementingType for DeclarationId {
    fn replace_implementing_type(&mut self, implementing_type: TyDeclaration) {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.replace_implementing_type(implementing_type);
        de_replace_decl_id(self.clone(), decl);
    }
}

impl DeclarationId {
    pub(crate) fn new(index: usize, span: Span) -> DeclarationId {
        DeclarationId(index, span)
    }

    pub(crate) fn with_parent(self, parent: DeclarationId) -> DeclarationId {
        de_register_parent(&self, parent);
        self
    }

    pub(crate) fn replace_id(&mut self, index: usize) {
        self.0 = index;
    }

    pub(crate) fn copy_types_and_insert_new(
        &self,
        type_mapping: &TypeMapping,
        type_engine: &TypeEngine,
    ) -> DeclarationId {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.copy_types(type_mapping, type_engine);
        de_insert(decl, self.1.clone()).with_parent(self.clone())
    }

    pub(crate) fn replace_self_type_and_insert_new(
        &self,
        type_engine: &TypeEngine,
        self_type: TypeId,
    ) -> DeclarationId {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.replace_self_type(type_engine, self_type);
        de_insert(decl, self.1.clone()).with_parent(self.clone())
    }

    pub(crate) fn replace_decls_and_insert_new(
        &self,
        decl_mapping: &DeclMapping,
        type_engine: &TypeEngine,
    ) -> DeclarationId {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.replace_decls(decl_mapping, type_engine);
        de_insert(decl, self.1.clone()).with_parent(self.clone())
    }
}
