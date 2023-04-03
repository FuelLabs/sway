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

use std::hash::{Hash, Hasher};

use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::ty::{
        self, TyAbiDecl, TyConstantDecl, TyEnumDecl, TyFunctionDecl, TyImplTrait, TyStorageDecl,
        TyStructDecl, TyTraitDecl, TyTraitFn,
    },
    type_system::*,
};

pub type DeclRefFunction = DeclRef<DeclId<TyFunctionDecl>>;
pub type DeclRefTrait = DeclRef<DeclId<TyTraitDecl>>;
pub type DeclRefTraitFn = DeclRef<DeclId<TyTraitFn>>;
pub type DeclRefImplTrait = DeclRef<DeclId<TyImplTrait>>;
pub type DeclRefStruct = DeclRef<DeclId<TyStructDecl>>;
pub type DeclRefStorage = DeclRef<DeclId<TyStorageDecl>>;
pub type DeclRefAbi = DeclRef<DeclId<TyAbiDecl>>;
pub type DeclRefConstant = DeclRef<DeclId<TyConstantDecl>>;
pub type DeclRefEnum = DeclRef<DeclId<TyEnumDecl>>;

pub type DeclRefMixedFunctional = DeclRef<FunctionalDeclId>;
pub type DeclRefMixedInterface = DeclRef<InterfaceDeclId>;

/// Represents the use of / syntactic reference to a declaration. A
/// smart-wrapper around a [DeclId], containing additional information about a
/// declaration.
#[derive(Debug, Clone)]
pub struct DeclRef<I> {
    /// The name of the declaration.
    // NOTE: In the case of storage, the name is "storage".
    name: Ident,

    /// The index into the [DeclEngine].
    id: I,

    /// The type substitution list to apply to the `id` field for type
    /// monomorphization.
    subst_list: SubstList,

    /// The [Span] of the entire declaration.
    decl_span: Span,
}

impl<I> DeclRef<I> {
    pub(crate) fn new(name: Ident, id: I, subst_list: SubstList, decl_span: Span) -> Self {
        DeclRef {
            name,
            id,
            subst_list,
            decl_span,
        }
    }

    pub(crate) fn into_parts(self) -> (Ident, I, SubstList, Span) {
        let DeclRef {
            name,
            id,
            subst_list,
            decl_span,
        } = self;
        (name, id, subst_list, decl_span)
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn id(&self) -> &I {
        &self.id
    }

    pub(crate) fn subst_list(&self) -> &SubstList {
        &self.subst_list
    }

    pub(crate) fn subst_list_mut(&mut self) -> &mut SubstList {
        &mut self.subst_list
    }

    pub fn decl_span(&self) -> &Span {
        &self.decl_span
    }

    pub(crate) fn fold<F, U>(self, f: F) -> U
    where
        F: Fn(I, SubstList) -> U,
    {
        f(self.id, self.subst_list)
    }
}

impl<T> DeclRef<DeclId<T>>
where
    T: SubstTypes + Named + Spanned,
    DeclEngine: DeclEngineGet<DeclId<T>, T>,
    DeclEngine: DeclEngineInsert<T>,
{
    pub(crate) fn relative_copy(&self, engines: Engines<'_>) -> Substituted<T> {
        let decl_engine = engines.de();
        self.clone()
            .fold_subst(engines)
            .map(|decl_ref| decl_engine.get(decl_ref.id()))
    }
}

impl<I> EqWithEngines for DeclRef<I> where I: Eq {}
impl<I> PartialEqWithEngines for DeclRef<I>
where
    I: PartialEq,
{
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let DeclRef {
            name: ln,
            id: lid,
            subst_list: lsl,
            // these fields are not used in comparison because they aren't
            // relevant/a reliable source of obj v. obj distinction
            decl_span: _,
        } = self;
        let DeclRef {
            name: rn,
            id: rid,
            subst_list: rsl,
            // these fields are not used in comparison because they aren't
            // relevant/a reliable source of obj v. obj distinction
            decl_span: _,
        } = other;
        ln == rn && lid == rid && lsl.eq(rsl, engines)
    }
}

impl<I> HashWithEngines for DeclRef<I>
where
    I: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let DeclRef {
            name,
            id,
            subst_list,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            decl_span: _,
        } = self;
        name.hash(state);
        id.hash(state);
        subst_list.hash(state, engines);
    }
}

impl<I> Spanned for DeclRef<I> {
    fn span(&self) -> Span {
        self.decl_span.clone()
    }
}

impl ReplaceFunctionImplementingType for DeclRefFunction {
    fn replace_implementing_type(&mut self, engines: Engines<'_>, implementing_type: ty::TyDecl) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(&self.id);
        decl.set_implementing_type(implementing_type);
        decl_engine.replace(self.id, decl);
    }
}

impl<T> CreateCopy<DeclRef<DeclId<T>>> for DeclRef<DeclId<T>> {
    fn scoped_copy(&self, engines: Engines<'_>) -> Self {
        DeclRef {
            name: self.name.clone(),
            id: self.id,
            subst_list: self.subst_list.scoped_copy(engines),
            decl_span: self.decl_span.clone(),
        }
    }

    fn unscoped_copy(&self) -> Self {
        self.clone()
    }
}

impl<T> From<DeclRef<DeclId<T>>> for DeclRef<InterfaceDeclId>
where
    InterfaceDeclId: From<DeclId<T>>,
{
    fn from(decl_ref: DeclRef<DeclId<T>>) -> Self {
        DeclRef {
            name: decl_ref.name,
            id: decl_ref.id.into(),
            subst_list: decl_ref.subst_list,
            decl_span: decl_ref.decl_span,
        }
    }
}
