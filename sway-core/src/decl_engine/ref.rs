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

use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::ty::{
        self, TyAbiDeclaration, TyConstantDeclaration, TyEnumDeclaration, TyFunctionDeclaration,
        TyImplTrait, TyStorageDeclaration, TyStructDeclaration, TyTraitDeclaration, TyTraitFn,
    },
    type_system::*,
};

pub type DeclRefFunction = DeclRef<DeclId<TyFunctionDeclaration>>;
pub type DeclRefTrait = DeclRef<DeclId<TyTraitDeclaration>>;
pub type DeclRefTraitFn = DeclRef<DeclId<TyTraitFn>>;
pub type DeclRefImplTrait = DeclRef<DeclId<TyImplTrait>>;
pub type DeclRefStruct = DeclRef<DeclId<TyStructDeclaration>>;
pub type DeclRefStorage = DeclRef<DeclId<TyStorageDeclaration>>;
pub type DeclRefAbi = DeclRef<DeclId<TyAbiDeclaration>>;
pub type DeclRefConstant = DeclRef<DeclId<TyConstantDeclaration>>;
pub type DeclRefEnum = DeclRef<DeclId<TyEnumDeclaration>>;

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
    subst_list: TypeSubstList,

    /// The [Span] of the entire declaration.
    decl_span: Span,
}

impl<I> DeclRef<I> {
    pub(crate) fn new(name: Ident, id: I, subst_list: TypeSubstList, decl_span: Span) -> Self {
        DeclRef {
            name,
            id,
            subst_list,
            decl_span,
        }
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn id(&self) -> &I {
        &self.id
    }

    pub(crate) fn subst_list(&self) -> &TypeSubstList {
        &self.subst_list
    }

    pub(crate) fn subst_list_mut(&mut self) -> &mut TypeSubstList {
        &mut self.subst_list
    }

    pub fn decl_span(&self) -> &Span {
        &self.decl_span
    }
}

impl<T> EqWithEngines for DeclRef<DeclId<T>> where T: PartialEq + Eq {}
impl<T> PartialEqWithEngines for DeclRef<T>
where
    T: PartialEq,
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

impl<T> HashWithEngines for DeclRef<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let DeclRef {
            id,
            subst_list,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            name: _,
            decl_span: _,
        } = self;
        id.hash(state);
        subst_list.hash(state, engines);
    }
}

impl<T> OrdWithEngines for DeclRef<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> std::cmp::Ordering {
        let DeclRef {
            id: lid,
            subst_list: lsl,
            // these fields are not used in comparison because they aren't
            // relevant/a reliable source of obj v. obj distinction
            name: _,
            decl_span: _,
        } = self;
        let DeclRef {
            id: rid,
            subst_list: rsl,
            // these fields are not used in comparison because they aren't
            // relevant/a reliable source of obj v. obj distinction
            name: _,
            decl_span: _,
        } = other;
        lid.cmp(rid).then_with(|| lsl.cmp(rsl, engines))
    }
}

impl<I> Spanned for DeclRef<I> {
    fn span(&self) -> Span {
        self.decl_span.clone()
    }
}

impl<I> SubstTypes for DeclRef<I> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let DeclRef {
            subst_list,
            // This field is excluded because DeclId's are unique and 1:1 with
            // declarations, so we do not create a new declaration or new DeclId
            // here. Moreover, all of the generic types for which we are calling
            // the `subst_inner` method are held inside of the `subst_list`
            // field.
            id: _,
            // these fields are excluded because they do not contain types
            name: _,
            decl_span: _,
        } = self;
        subst_list.subst(type_mapping, engines);
    }
}

impl ReplaceFunctionImplementingType for DeclRefFunction {
    fn replace_implementing_type(
        &mut self,
        engines: Engines<'_>,
        implementing_type: ty::TyDeclaration,
    ) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self.id);
        decl.set_implementing_type(implementing_type);
        decl_engine.replace(self.id, decl);
    }
}
