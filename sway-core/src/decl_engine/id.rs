use sway_types::{Ident, Span, Spanned};

use crate::{engine_threading::*, language::ty, type_system::*};

use super::{DeclEngine, DeclMapping, ReplaceDecls, ReplaceFunctionImplementingType};

/// An ID used to refer to an item in the [DeclEngine](super::decl_engine::DeclEngine)
#[derive(Debug)]
pub struct DeclId {
    /// The name of the declaration.
    // NOTE: In the case of storage, the name is "storage".
    pub name: Ident,

    /// The low-level index into the [DeclEngine].
    id: usize,

    /// The [Span] of the entire declaration.
    decl_span: Span,
}

impl Clone for DeclId {
    fn clone(&self) -> DeclId {
        DeclId {
            name: self.name.clone(),
            id: self.id,
            decl_span: self.decl_span.clone(),
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl EqWithEngines for DeclId {}
impl PartialEqWithEngines for DeclId {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let decl_engine = engines.de();
        let left = decl_engine.get(self.clone());
        let right = decl_engine.get(other.clone());
        left.eq(&right, engines)
    }
}

impl std::ops::Deref for DeclId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

#[allow(clippy::from_over_into)]
impl Into<usize> for DeclId {
    fn into(self) -> usize {
        self.id
    }
}

impl Spanned for DeclId {
    fn span(&self) -> Span {
        self.decl_span.clone()
    }
}

impl SubstTypes for DeclId {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.subst(type_mapping, engines);
        decl_engine.replace(self.clone(), decl);
    }
}

impl ReplaceSelfType for DeclId {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(self.clone(), decl);
    }
}

impl ReplaceDecls for DeclId {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        let decl_engine = engines.de();
        if let Some(new_decl_id) = decl_mapping.find_match(self) {
            self.id = *new_decl_id;
            return;
        }
        let all_parents = decl_engine.find_all_parents(engines, self.clone());
        for parent in all_parents.into_iter() {
            if let Some(new_decl_id) = decl_mapping.find_match(&parent) {
                self.id = *new_decl_id;
                return;
            }
        }
    }
}

impl ReplaceFunctionImplementingType for DeclId {
    fn replace_implementing_type(
        &mut self,
        engines: Engines<'_>,
        implementing_type: ty::TyDeclaration,
    ) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.replace_implementing_type(engines, implementing_type);
        decl_engine.replace(self.clone(), decl);
    }
}

impl DeclId {
    pub(crate) fn new(name: Ident, id: usize, decl_span: Span) -> DeclId {
        DeclId {
            name,
            id,
            decl_span,
        }
    }

    pub(crate) fn with_parent(self, decl_engine: &DeclEngine, parent: DeclId) -> DeclId {
        decl_engine.register_parent(&self, parent);
        self
    }

    pub(crate) fn replace_id(&mut self, index: usize) {
        self.id = index;
    }

    pub(crate) fn subst_types_and_insert_new(
        &self,
        type_mapping: &TypeSubstMap,
        engines: Engines<'_>,
    ) -> DeclId {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.subst(type_mapping, engines);
        decl_engine
            .insert_wrapper(self.name.clone(), decl, self.decl_span.clone())
            .with_parent(decl_engine, self.clone())
    }

    pub(crate) fn replace_self_type_and_insert_new(
        &self,
        engines: Engines<'_>,
        self_type: TypeId,
    ) -> DeclId {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.replace_self_type(engines, self_type);
        decl_engine
            .insert_wrapper(self.name.clone(), decl, self.decl_span.clone())
            .with_parent(decl_engine, self.clone())
    }

    pub(crate) fn replace_decls_and_insert_new(
        &self,
        decl_mapping: &DeclMapping,
        engines: Engines<'_>,
    ) -> DeclId {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.clone());
        decl.replace_decls(decl_mapping, engines);
        decl_engine
            .insert_wrapper(self.name.clone(), decl, self.decl_span.clone())
            .with_parent(decl_engine, self.clone())
    }
}
