use std::marker::PhantomData;
use std::{fmt, hash::Hash};

use crate::{
    decl_engine::*,
    engine_threading::*,
    language::ty::{
        TyEnumDeclaration, TyFunctionDeclaration, TyImplTrait, TyStructDeclaration,
        TyTraitDeclaration, TyTraitFn, TyTypeAliasDeclaration,
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
        Self(self.0, PhantomData)
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
        self.0.partial_cmp(&other.0)
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
}

#[allow(clippy::from_over_into)]
impl<T> Into<usize> for DeclId<T> {
    fn into(self) -> usize {
        self.0
    }
}

impl SubstTypes for DeclId<TyFunctionDeclaration> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyTraitDeclaration> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyTraitFn> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyImplTrait> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyStructDeclaration> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyEnumDeclaration> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}
impl SubstTypes for DeclId<TyTypeAliasDeclaration> {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(*self, decl);
    }
}

impl ReplaceSelfType for DeclId<TyFunctionDeclaration> {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyTraitDeclaration> {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyTraitFn> {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyImplTrait> {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyStructDeclaration> {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyEnumDeclaration> {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
impl ReplaceSelfType for DeclId<TyTypeAliasDeclaration> {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(*self, decl);
    }
}
