use sway_types::{Ident, Span, Spanned};

use crate::{decl_engine::*, engine_threading::*, language::ty, type_system::*};

/// A smart-wrapper around a [DeclId], containing additional information about
/// a declaration.
#[derive(Debug, Clone)]
pub struct DeclRef {
    /// The name of the declaration.
    // NOTE: In the case of storage, the name is "storage".
    pub name: Ident,

    /// The index into the [DeclEngine].
    pub(crate) id: DeclId,

    /// The [Span] of the entire declaration.
    decl_span: Span,
}

impl DeclRef {
    pub(crate) fn new(name: Ident, id: usize, decl_span: Span) -> DeclRef {
        DeclRef {
            name,
            id: DeclId::new(id),
            decl_span,
        }
    }

    pub(crate) fn with_parent(self, decl_engine: &DeclEngine, parent: DeclRef) -> DeclRef {
        decl_engine.register_parent(&self, parent);
        self
    }

    pub(crate) fn replace_id(&mut self, index: DeclId) {
        self.id.replace_id(index);
    }

    pub(crate) fn subst_types_and_insert_new(
        &self,
        type_mapping: &TypeSubstMap,
        engines: Engines<'_>,
    ) -> DeclRef {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine
            .insert_wrapper(self.name.clone(), decl, self.decl_span.clone())
            .with_parent(decl_engine, self.clone())
    }

    pub(crate) fn replace_self_type_and_insert_new(
        &self,
        engines: Engines<'_>,
        self_type: TypeId,
    ) -> DeclRef {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine
            .insert_wrapper(self.name.clone(), decl, self.decl_span.clone())
            .with_parent(decl_engine, self.clone())
    }

    pub(crate) fn replace_decls_and_insert_new(
        &self,
        decl_mapping: &DeclMapping,
        engines: Engines<'_>,
    ) -> DeclRef {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(&self.clone());
        decl.replace_decls(decl_mapping, engines);
        decl_engine
            .insert_wrapper(self.name.clone(), decl, self.decl_span.clone())
            .with_parent(decl_engine, self.clone())
    }
}

impl EqWithEngines for DeclRef {}
impl PartialEqWithEngines for DeclRef {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let decl_engine = engines.de();
        let left = decl_engine.get(self);
        let right = decl_engine.get(other);
        self.name == other.name && left.eq(&right, engines)
    }
}

impl Spanned for DeclRef {
    fn span(&self) -> Span {
        self.decl_span.clone()
    }
}

impl SubstTypes for DeclRef {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(self, decl);
    }
}

impl ReplaceSelfType for DeclRef {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(self, decl);
    }
}

impl ReplaceDecls for DeclRef {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        let decl_engine = engines.de();
        if let Some(new_decl_ref) = decl_mapping.find_match(self) {
            self.id = new_decl_ref.id;
            return;
        }
        let all_parents = decl_engine.find_all_parents(engines, self);
        for parent in all_parents.into_iter() {
            if let Some(new_decl_ref) = decl_mapping.find_match(&parent) {
                self.id = new_decl_ref.id;
                return;
            }
        }
    }
}

impl ReplaceFunctionImplementingType for DeclRef {
    fn replace_implementing_type(
        &mut self,
        engines: Engines<'_>,
        implementing_type: ty::TyDeclaration,
    ) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_implementing_type(engines, implementing_type);
        decl_engine.replace(self, decl);
    }
}
