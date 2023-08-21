use std::marker::PhantomData;
use std::{fmt, hash::Hash};

use crate::language::ty::TyTraitType;
use crate::{
    decl_engine::*,
    engine_threading::*,
    language::ty::{
        TyEnumDecl, TyFunctionDecl, TyImplTrait, TyStructDecl, TyTraitDecl, TyTraitFn,
        TyTypeAliasDecl,
    },
    type_system::*,
};

/// An ID used to refer to an item in the [DeclEngine](super::decl_engine::DeclEngine)
pub struct DeclId<T>(usize, PhantomData<T>);

impl<T> fmt::Debug for DeclId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DeclId").field(&self.0).finish()
    }
}

impl<T> DeclId<T> {
    pub(crate) fn inner(&self) -> usize {
        self.0
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

impl SubstTypes for DeclId<TyFunctionDecl> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyTraitDecl> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyTraitFn> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyImplTrait> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyStructDecl> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyEnumDecl> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyTypeAliasDecl> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyTraitType> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}

impl ReplaceSelfType for DeclId<TyFunctionDecl> {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyTraitDecl> {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyTraitFn> {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyImplTrait> {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyStructDecl> {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyEnumDecl> {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyTypeAliasDecl> {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyTraitType> {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
