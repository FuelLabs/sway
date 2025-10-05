use crate::{
    decl_engine::*,
    engine_threading::*,
    language::ty::{
        TyConstantDecl, TyDeclParsedType, TyEnumDecl, TyFunctionDecl, TyImplSelfOrTrait,
        TyStructDecl, TyTraitDecl, TyTraitFn, TyTraitType, TyTypeAliasDecl,
    },
    type_system::*,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use sway_types::{Named, Spanned};

pub type DeclIdIndexType = usize;

/// An ID used to refer to an item in the [DeclEngine](super::decl_engine::DeclEngine)
pub struct DeclId<T>(DeclIdIndexType, PhantomData<T>);

impl<T> fmt::Debug for DeclId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DeclId").field(&self.0).finish()
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct DeclUniqueId(pub(crate) u64);

impl<T> DeclId<T> {
    pub(crate) fn inner(&self) -> DeclIdIndexType {
        self.0
    }

    pub fn unique_id(&self) -> DeclUniqueId
    where
        T: 'static,
    {
        let mut hasher = DefaultHasher::default();
        std::any::TypeId::of::<T>().hash(&mut hasher);
        self.0.hash(&mut hasher);

        DeclUniqueId(hasher.finish())
    }
}

impl<T> Copy for DeclId<T> {}
impl<T> Clone for DeclId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for DeclId<T> {}
impl<T> Hash for DeclId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
impl<T> PartialEq for DeclId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T> PartialOrd for DeclId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for DeclId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> DeclId<T> {
    pub(crate) fn new(id: usize) -> Self {
        DeclId(id, PhantomData)
    }

    pub(crate) fn replace_id(&mut self, index: Self) {
        self.0 = index.0;
    }

    pub(crate) fn dummy() -> Self {
        // we assume that `usize::MAX` id is not possible in practice
        Self(usize::MAX, PhantomData)
    }
}

#[allow(clippy::from_over_into)]
impl<T> Into<usize> for DeclId<T> {
    fn into(self) -> usize {
        self.0
    }
}

impl<T> DebugWithEngines for DeclId<T>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + DebugWithEngines,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let decl = engines.de().get(self);
        DebugWithEngines::fmt(&decl, f, engines)
    }
}

impl<T> EqWithEngines for DeclId<T>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + PartialEqWithEngines + EqWithEngines,
{
}

impl<T> PartialEqWithEngines for DeclId<T>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + PartialEqWithEngines,
{
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        let decl_engine = ctx.engines().de();
        let l_decl = decl_engine.get(self);
        let r_decl = decl_engine.get(other);
        l_decl.name() == r_decl.name() && l_decl.eq(&r_decl, ctx)
    }
}

impl<T> HashWithEngines for DeclId<T>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned + HashWithEngines,
{
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let decl_engine = engines.de();
        let decl = decl_engine.get(self);
        decl.name().hash(state);
        decl.hash(state, engines);
    }
}

impl SubstTypes for DeclId<TyFunctionDecl> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(self)).clone();
        if decl.subst(ctx).has_changes() {
            decl_engine.replace(*self, decl);
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}
impl SubstTypes for DeclId<TyTraitDecl> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(self)).clone();
        if decl.subst(ctx).has_changes() {
            decl_engine.replace(*self, decl);
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}
impl SubstTypes for DeclId<TyTraitFn> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(self)).clone();
        if decl.subst(ctx).has_changes() {
            decl_engine.replace(*self, decl);
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}
impl SubstTypes for DeclId<TyImplSelfOrTrait> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(self)).clone();
        if decl.subst(ctx).has_changes() {
            decl_engine.replace(*self, decl);
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}
impl SubstTypes for DeclId<TyStructDecl> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(self)).clone();
        if decl.subst(ctx).has_changes() {
            decl_engine.replace(*self, decl);
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}
impl SubstTypes for DeclId<TyEnumDecl> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(self)).clone();
        if decl.subst(ctx).has_changes() {
            decl_engine.replace(*self, decl);
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}
impl SubstTypes for DeclId<TyTypeAliasDecl> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(self)).clone();
        if decl.subst(ctx).has_changes() {
            decl_engine.replace(*self, decl);
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}

impl SubstTypes for DeclId<TyTraitType> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(self)).clone();
        if decl.subst(ctx).has_changes() {
            decl_engine.replace(*self, decl);
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}

impl SubstTypes for DeclId<TyConstantDecl> {
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(self)).clone();
        if decl.subst(ctx).has_changes() {
            *self = *decl_engine.insert(decl, None).id();
            HasChanges::Yes
        } else {
            HasChanges::No
        }
    }
}

impl<T> DeclId<T>
where
    DeclEngine: DeclEngineIndex<T> + DeclEngineInsert<T> + DeclEngineGetParsedDeclId<T>,
    T: Named + Spanned + SubstTypes + Clone + TyDeclParsedType,
{
    pub(crate) fn subst_types_and_insert_new(
        &self,
        ctx: &SubstTypesContext,
    ) -> Option<DeclRef<Self>> {
        let decl_engine = ctx.engines.de();
        let mut decl = (*decl_engine.get(self)).clone();
        if decl.subst(ctx).has_changes() {
            Some(decl_engine.insert(decl, decl_engine.get_parsed_decl_id(self).as_ref()))
        } else {
            None
        }
    }
}

impl<T> Serialize for DeclId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for DeclId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = DeclIdIndexType::deserialize(deserializer)?;
        Ok(DeclId::new(id))
    }
}
