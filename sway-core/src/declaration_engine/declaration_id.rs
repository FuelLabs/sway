use std::fmt;

use sway_types::{Span, Spanned};

use crate::{
    type_system::{CopyTypes, TypeMapping},
    ReplaceSelfType, TypeId,
};

use super::{
    de_find_all_parents, de_insert, de_register_parent,
    declaration_engine::{de_look_up_decl_id, de_replace_decl_id},
    DeclMapping, ReplaceDecls,
};

/// An ID used to refer to an item in the [DeclarationEngine](super::declaration_engine::DeclarationEngine)
#[derive(Debug, Eq)]
pub struct DeclarationId(usize, Span);

impl Clone for DeclarationId {
    fn clone(&self) -> Self {
        Self(self.0, self.1.clone())
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for DeclarationId {
    fn eq(&self, other: &Self) -> bool {
        de_look_up_decl_id(self.clone()) == de_look_up_decl_id(other.clone())
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
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping) {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.copy_types(type_mapping);
        de_replace_decl_id(self.clone(), decl);
    }
}

impl ReplaceSelfType for DeclarationId {
    fn replace_self_type(&mut self, self_type: TypeId) {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.replace_self_type(self_type);
        de_replace_decl_id(self.clone(), decl);
    }
}

impl ReplaceDecls for DeclarationId {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping) {
        if let Some(new_decl_id) = decl_mapping.find_match(self) {
            println!("switching {} and {}", self.0, *new_decl_id);
            self.0 = *new_decl_id;
            return;
        }
        let all_parents = de_find_all_parents(self.clone());
        println!(
            "self: {}, all_parents: [{}]",
            **self,
            all_parents
                .iter()
                .map(|x| format!("{}", **x))
                .collect::<Vec<_>>()
                .join(", ")
        );
        for parent in all_parents.into_iter() {
            if let Some(new_decl_id) = decl_mapping.find_match(&parent) {
                println!("switching {} and {}", self.0, *new_decl_id);
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

    pub(crate) fn with_parent(self, parent: DeclarationId) -> DeclarationId {
        de_register_parent(&self, parent);
        self
    }

    pub(crate) fn replace_id(&mut self, index: usize) {
        self.0 = index;
    }

    pub(crate) fn copy_types_and_insert_new(&self, type_mapping: &TypeMapping) -> DeclarationId {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.copy_types(type_mapping);
        de_insert(decl, self.1.clone())
    }

    pub(crate) fn replace_self_type_and_insert_new(&self, self_type: TypeId) -> DeclarationId {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.replace_self_type(self_type);
        de_insert(decl, self.1.clone())
    }
}
