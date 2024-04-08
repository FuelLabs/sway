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

use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Ident, Named, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::ty::{
        self, TyAbiDecl, TyConstantDecl, TyEnumDecl, TyFunctionDecl, TyImplTrait, TyStorageDecl,
        TyStructDecl, TyTraitDecl, TyTraitFn, TyTraitType,
    },
    semantic_analysis::TypeCheckContext,
    type_system::*,
};

pub type DeclRefFunction = DeclRef<DeclId<TyFunctionDecl>>;
pub type DeclRefTrait = DeclRef<DeclId<TyTraitDecl>>;
pub type DeclRefTraitFn = DeclRef<DeclId<TyTraitFn>>;
pub type DeclRefTraitType = DeclRef<DeclId<TyTraitType>>;
pub type DeclRefImplTrait = DeclRef<DeclId<TyImplTrait>>;
pub type DeclRefStruct = DeclRef<DeclId<TyStructDecl>>;
pub type DeclRefStorage = DeclRef<DeclId<TyStorageDecl>>;
pub type DeclRefAbi = DeclRef<DeclId<TyAbiDecl>>;
pub type DeclRefConstant = DeclRef<DeclId<TyConstantDecl>>;
pub type DeclRefEnum = DeclRef<DeclId<TyEnumDecl>>;

pub type DeclRefMixedFunctional = DeclRef<AssociatedItemDeclId>;
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
    pub(crate) fn new(name: Ident, id: I, decl_span: Span) -> Self {
        DeclRef {
            name,
            id,
            subst_list: SubstList::new(),
            decl_span,
        }
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

    pub fn decl_span(&self) -> &Span {
        &self.decl_span
    }
}

impl<T> DeclRef<DeclId<T>> {
    pub(crate) fn replace_id(&mut self, index: DeclId<T>) {
        self.id.replace_id(index);
    }

    pub fn start_subs_type(&self) -> SubstTypesRefInProgress<'_, DeclRef<DeclId<T>>, T>
    where
        T: Clone,
    {
        SubstTypesRefInProgress {
            original_ref: self,
            new_decl: None,
        }
    }
}

#[derive(Clone)]
pub struct SubstTypesRefInProgress<'a, TRef: Clone, TDecl> {
    pub original_ref: &'a TRef,
    pub new_decl: Option<TDecl>,
}

impl<'a, T> SubstTypesRefInProgress<'a, DeclRef<DeclId<T>>, T> {
    pub fn replace(self, engines: &Engines) -> DeclRef<DeclId<T>>
    where
        DeclEngine: DeclEngineReplace<T>,
    {
        match self.new_decl {
            Some(new_decl) => {
                engines.de().replace(*self.original_ref.id(), new_decl);
                self.original_ref.clone()
            }
            None => self.original_ref.clone(),
        }
    }

    pub fn insert_new(self, engines: &Engines) -> DeclRef<DeclId<T>>
    where
        T: Named + Spanned,
        DeclEngine: DeclEngineInsert<T>,
    {
        match self.new_decl {
            Some(new_decl) => engines.de().insert(new_decl),
            None => self.original_ref.clone(),
        }
    }

    pub fn insert_new_with_parent(self, engines: &Engines) -> DeclRef<DeclId<T>>
    where
        AssociatedItemDeclId: From<DeclId<T>>,
        T: Named + Spanned,
        DeclEngine: DeclEngineInsert<T>,
    {
        match self.new_decl {
            Some(new_decl) => engines
                .de()
                .insert(new_decl)
                .with_parent(engines.de(), (*self.original_ref.id()).into()),
            None => self.original_ref.clone(),
        }
    }
}

impl<'a> SubstTypesRefInProgress<'a, AssociatedItemDeclId, AssociatedItemDecl> {
    pub fn replace(self, engines: &Engines) -> AssociatedItemDeclId {
        use AssociatedItemDecl as B;
        use AssociatedItemDeclId as A;
        match (self.original_ref, self.new_decl) {
            (A::TraitFn(id), Some(B::TraitFn(new_decl))) => {
                engines.de().replace(*id, new_decl);
                self.original_ref.clone()
            }
            (A::Function(id), Some(B::Function(new_decl))) => {
                engines.de().replace(*id, new_decl);
                self.original_ref.clone()
            }
            (A::Constant(id), Some(B::Constant(new_decl))) => {
                engines.de().replace(*id, new_decl);
                self.original_ref.clone()
            }
            (A::Type(id), Some(B::Type(new_decl))) => {
                engines.de().replace(*id, new_decl);
                self.original_ref.clone()
            }
            _ => unreachable!(),
        }
    }

    pub fn insert_new(self, engines: &Engines) -> AssociatedItemDeclId {
        use AssociatedItemDecl as B;
        use AssociatedItemDeclId as A;
        match (self.original_ref, self.new_decl) {
            (A::TraitFn(id), Some(B::TraitFn(new_decl))) => {
                AssociatedItemDeclId::TraitFn(engines.de().insert(new_decl).id().clone())
            }
            (A::Function(id), Some(B::Function(new_decl))) => {
                AssociatedItemDeclId::Function(engines.de().insert(new_decl).id().clone())
            }
            (A::Constant(id), Some(B::Constant(new_decl))) => {
                AssociatedItemDeclId::Constant(engines.de().insert(new_decl).id().clone())
            }
            (A::Type(id), Some(B::Type(new_decl))) => {
                AssociatedItemDeclId::Type(engines.de().insert(new_decl).id().clone())
            }
            _ => unreachable!(),
        }
    }

    pub fn insert_new_with_parent(self, engines: &Engines) -> AssociatedItemDeclId {
        use AssociatedItemDecl as B;
        use AssociatedItemDeclId as A;
        match (self.original_ref, self.new_decl) {
            (A::TraitFn(id), Some(B::TraitFn(new_decl))) => AssociatedItemDeclId::TraitFn(
                engines
                    .de()
                    .insert(new_decl)
                    .with_parent(engines.de(), (*id).into())
                    .id()
                    .clone()
                    .into(),
            ),
            (A::Function(id), Some(B::Function(new_decl))) => AssociatedItemDeclId::Function(
                engines
                    .de()
                    .insert(new_decl)
                    .with_parent(engines.de(), (*id).into())
                    .id()
                    .clone()
                    .into(),
            ),
            (A::Constant(id), Some(B::Constant(new_decl))) => AssociatedItemDeclId::Constant(
                engines
                    .de()
                    .insert(new_decl)
                    .with_parent(engines.de(), (*id).into())
                    .id()
                    .clone()
                    .into(),
            ),
            (A::Type(id), Some(B::Type(new_decl))) => AssociatedItemDeclId::Type(
                engines
                    .de()
                    .insert(new_decl)
                    .with_parent(engines.de(), (*id).into())
                    .id()
                    .clone()
                    .into(),
            ),
            _ => unreachable!(),
        }
    }
}

impl<'a, T> SubstTypes for SubstTypesRefInProgress<'a, DeclRef<DeclId<T>>, T>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + SubstTypes + Clone,
{
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        let decl = engines.de().get(self.original_ref.id());
        let new_decl = decl.subst(type_mapping, engines)?;
        Some(Self {
            original_ref: self.original_ref,
            new_decl: Some(new_decl),
        })
    }
}

#[derive(Clone)]
pub enum AssociatedItemDecl {
    TraitFn(TyTraitFn),
    Function(TyFunctionDecl),
    Constant(TyConstantDecl),
    Type(TyTraitType),
}

impl<'a> SubstTypes for SubstTypesRefInProgress<'a, AssociatedItemDeclId, AssociatedItemDecl> {
    fn subst_inner(&self, type_mapping: &TypeSubstMap, engines: &Engines) -> Option<Self> {
        match self.original_ref {
            AssociatedItemDeclId::TraitFn(id) => {
                let decl = engines.de().get(id);
                let decl = decl.subst(type_mapping, engines)?;
                Some(Self {
                    original_ref: self.original_ref,
                    new_decl: Some(AssociatedItemDecl::TraitFn(decl)),
                })
            }
            AssociatedItemDeclId::Function(id) => {
                let decl = engines.de().get(id);
                let decl = decl.subst(type_mapping, engines)?;
                Some(Self {
                    original_ref: self.original_ref,
                    new_decl: Some(AssociatedItemDecl::Function(decl)),
                })
            }
            AssociatedItemDeclId::Constant(id) => {
                let decl = engines.de().get(id);
                let decl = decl.subst(type_mapping, engines)?;
                Some(Self {
                    original_ref: self.original_ref,
                    new_decl: Some(AssociatedItemDecl::Constant(decl)),
                })
            }
            AssociatedItemDeclId::Type(id) => {
                let decl = engines.de().get(id);
                let decl = decl.subst(type_mapping, engines)?;
                Some(Self {
                    original_ref: self.original_ref,
                    new_decl: Some(AssociatedItemDecl::Type(decl)),
                })
            }
        }
    }
}

// impl<T> DeclRef<DeclId<T>>
// where
//     DeclEngine: DeclEngineIndex<T>,
//     T: Named + Spanned + SubstTypes + Clone,
// {
//     pub(crate) fn subst_types_and_insert_new(
//         &self,
//         type_mapping: &TypeSubstMap,
//         engines: &Engines,
//     ) -> Option<Self> {
//         let decl_engine = engines.de();
//         let mut decl = decl_engine.get(&self.id);
//         if let Some(decl) = decl.subst(type_mapping, engines) {
//             Some(decl_engine.insert(decl))
//         } else {
//             None
//         }
//     }
// }

impl<T> DeclRef<DeclId<T>>
where
    AssociatedItemDeclId: From<DeclId<T>>,
{
    pub(crate) fn with_parent(
        self,
        decl_engine: &DeclEngine,
        parent: AssociatedItemDeclId,
    ) -> Self {
        let id: DeclId<T> = self.id;
        decl_engine.register_parent(id.into(), parent.clone());
        self
    }
}

impl<T> DeclRef<DeclId<T>>
where
    AssociatedItemDeclId: From<DeclId<T>>,
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + ReplaceDecls + std::fmt::Debug + Clone,
{
    /// Returns Ok(None), if nothing was replaced.
    /// Ok(true), if something was replaced.
    /// and errors when appropriated.
    pub(crate) fn replace_decls_and_insert_new_with_parent(
        &self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<Option<Self>, ErrorEmitted> {
        let decl_engine = ctx.engines().de();

        let original = decl_engine.get(&self.id);

        let mut new = (*original).clone();
        let changed = new.replace_decls(decl_mapping, handler, ctx)?;

        Ok(changed.then(|| {
            decl_engine
                .insert(new)
                .with_parent(decl_engine, self.id.into())
        }))
    }
}

impl<T> EqWithEngines for DeclRef<DeclId<T>>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + PartialEqWithEngines + EqWithEngines,
{
}
impl<T> PartialEqWithEngines for DeclRef<DeclId<T>>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + PartialEqWithEngines,
{
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let decl_engine = ctx.engines().de();
        let DeclRef {
            name: ln,
            id: lid,
            // these fields are not used in comparison because they aren't
            // relevant/a reliable source of obj v. obj distinction
            decl_span: _,
            // temporarily omitted
            subst_list: _,
        } = self;
        let DeclRef {
            name: rn,
            id: rid,
            // these fields are not used in comparison because they aren't
            // relevant/a reliable source of obj v. obj distinction
            decl_span: _,
            // temporarily omitted
            subst_list: _,
        } = other;
        ln == rn && decl_engine.get(lid).eq(&decl_engine.get(rid), ctx)
    }
}

impl<T> HashWithEngines for DeclRef<DeclId<T>>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + HashWithEngines,
{
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let decl_engine = engines.de();
        let DeclRef {
            name,
            id,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            decl_span: _,
            // temporarily omitted
            subst_list: _,
        } = self;
        name.hash(state);
        decl_engine.get(id).hash(state, engines);
    }
}

impl EqWithEngines for DeclRefMixedInterface {}
impl PartialEqWithEngines for DeclRefMixedInterface {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let decl_engine = ctx.engines().de();
        match (&self.id, &other.id) {
            (InterfaceDeclId::Abi(self_id), InterfaceDeclId::Abi(other_id)) => {
                let left = decl_engine.get(self_id);
                let right = decl_engine.get(other_id);
                self.name == other.name && left.eq(&right, ctx)
            }
            (InterfaceDeclId::Trait(self_id), InterfaceDeclId::Trait(other_id)) => {
                let left = decl_engine.get(self_id);
                let right = decl_engine.get(other_id);
                self.name == other.name && left.eq(&right, ctx)
            }
            _ => false,
        }
    }
}

impl HashWithEngines for DeclRefMixedInterface {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        match self.id {
            InterfaceDeclId::Abi(id) => {
                state.write_u8(0);
                let decl_engine = engines.de();
                let decl = decl_engine.get(&id);
                decl.hash(state, engines);
            }
            InterfaceDeclId::Trait(id) => {
                state.write_u8(1);
                let decl_engine = engines.de();
                let decl = decl_engine.get(&id);
                decl.hash(state, engines);
            }
        }
    }
}

impl<I> Spanned for DeclRef<I> {
    fn span(&self) -> Span {
        self.decl_span.clone()
    }
}

impl ReplaceDecls for DeclRefFunction {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
    ) -> Result<bool, ErrorEmitted> {
        let engines = ctx.engines();
        let decl_engine = engines.de();

        let func = decl_engine.get(self);

        if let Some(new_decl_ref) = decl_mapping.find_match(
            handler,
            ctx.engines(),
            self.id.into(),
            func.implementing_for_typeid,
            ctx.self_type(),
        )? {
            return Ok(
                if let AssociatedItemDeclId::Function(new_decl_ref) = new_decl_ref {
                    self.id = new_decl_ref;
                    true
                } else {
                    false
                },
            );
        }
        let all_parents = decl_engine.find_all_parents(engines, &self.id);
        for parent in all_parents.iter() {
            if let Some(new_decl_ref) = decl_mapping.find_match(
                handler,
                ctx.engines(),
                parent.clone(),
                func.implementing_for_typeid,
                ctx.self_type(),
            )? {
                return Ok(
                    if let AssociatedItemDeclId::Function(new_decl_ref) = new_decl_ref {
                        self.id = new_decl_ref;
                        true
                    } else {
                        false
                    },
                );
            }
        }
        Ok(false)
    }
}

impl ReplaceFunctionImplementingType for DeclRefFunction {
    fn replace_implementing_type(&mut self, engines: &Engines, implementing_type: ty::TyDecl) {
        let decl_engine = engines.de();
        let mut decl = (*decl_engine.get(&self.id)).clone();
        decl.set_implementing_type(implementing_type);
        decl_engine.replace(self.id, decl);
    }
}
