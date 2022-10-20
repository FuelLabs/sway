use std::fmt;

use sway_types::{Span, Spanned};

use crate::type_system::{CopyTypes, TypeMapping};

use super::{
    de_insert_function,
    declaration_engine::{de_look_up_decl_id, de_replace_decl_id},
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
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        if type_mapping.is_empty() {
            return;
        }
        let mut decl = de_look_up_decl_id(self.clone());
        decl.copy_types(type_mapping);
        de_replace_decl_id(self.clone(), decl);
    }
}

impl DeclarationId {
    pub(super) fn new(index: usize, span: Span) -> DeclarationId {
        DeclarationId(index, span)
    }

    pub(crate) fn replace_id(&mut self, index: usize) {
        self.0 = index;
    }

    pub(crate) fn copy_and_insert_new(&self, type_mapping: &TypeMapping) -> DeclarationId {
        let mut decl = de_look_up_decl_id(self.clone());
        decl.copy_types(type_mapping);
        let function = decl.expect_function(&self.1).unwrap();
        de_insert_function(function)
    }
}
