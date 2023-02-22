//! Represents the use of / syntactic reference to a declaration.
//!
//! ### Is a [DeclRef] effectively the same as a [DeclId]?
//!
//! A [DeclRef] is a smart wrapper around a [DeclId] and canonically represents
//! the use / syntactic reference to a declaration. This does not include the
//! syntactic locations for where declarations are declared though. For example,
//! function declaration `fn my_function() { .. }` would just create a [DeclId],
//! while function application `my_function()` would create a [DeclRef].
//!
//! [DeclRef] contains a [DeclId] field `id`, as well as some additional helpful
//! information. These additional fields include an [Ident] for the declaration
//! `name` and a [Span] for the declaration `decl_span`. Note, `name` and
//! `decl_span` can also be found by using `id` to get the declaration itself
//! from the [DeclEngine]. But the [DeclRef] type allows Sway compiler writers
//! to reduce unnecessary lookups into the [DeclEngine] when only the `name` or
//! `decl_span` is desired.
//!
//! It is recommend to use [DeclId] for cases like function declaration
//! `fn my_function() { .. }`, and to use [DeclRef] for cases like function
//! application `my_function()`.

use std::hash::Hasher;

use sway_types::{Ident, Span, Spanned};

use crate::{decl_engine::*, engine_threading::*, language::ty, type_system::*};

/// Represents the use of / syntactic reference to a declaration. A
/// smart-wrapper around a [DeclId], containing additional information about a
/// declaration.
#[derive(Debug, Clone)]
pub struct DeclRef {
    /// The name of the declaration.
    // NOTE: In the case of storage, the name is "storage".
    pub name: Ident,

    /// The index into the [DeclEngine].
    pub id: DeclId,

    /// The [Span] of the entire declaration.
    pub decl_span: Span,
}

impl DeclRef {
    pub(crate) fn new(name: Ident, id: usize, decl_span: Span) -> DeclRef {
        DeclRef {
            name,
            id: DeclId::new(id),
            decl_span,
        }
    }

    pub(crate) fn with_parent<'a, T>(self, decl_engine: &DeclEngine, parent: &'a T) -> DeclRef
    where
        DeclId: From<&'a T>,
    {
        decl_engine.register_parent::<T>(&self, parent);
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
            .with_parent(decl_engine, self)
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
            .with_parent(decl_engine, self)
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

impl HashWithEngines for DeclRef {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let decl = decl_engine.get(self);
        decl.hash(state, engines);
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

impl ReplaceDecls for DeclRef {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        let decl_engine = engines.de();
        if let Some(new_decl_ref) = decl_mapping.find_match(self) {
            self.id = new_decl_ref;
            return;
        }
        let all_parents = decl_engine.find_all_parents(engines, self);
        for parent in all_parents.iter() {
            if let Some(new_decl_ref) = decl_mapping.find_match(parent) {
                self.id = new_decl_ref;
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
