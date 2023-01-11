use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    sync::RwLock,
};

use sway_error::error::CompileError;
use sway_types::{Span, Spanned};

use crate::{
    concurrent_slab::{ConcurrentSlab, ListDisplay},
    declaration_engine::*,
    engine_threading::*,
    language::ty,
};

/// Used inside of type inference to store declarations.
#[derive(Debug, Default)]
pub struct DeclEngine {
    slab: ConcurrentSlab<DeclWrapper>,
    parents: RwLock<HashMap<usize, Vec<DeclId>>>,
}

impl Clone for DeclEngine {
    fn clone(&self) -> Self {
        let parents = self.parents.read().unwrap();
        DeclEngine {
            slab: self.slab.clone(),
            parents: RwLock::new(parents.clone()),
        }
    }
}

impl fmt::Display for DeclEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.slab.with_slice(|elems| {
            let list = ListDisplay { list: elems.iter() };
            write!(f, "DeclarationEngine {{\n{}\n}}", list)
        })
    }
}

impl DeclEngine {
    pub(crate) fn get(&self, index: DeclId) -> DeclWrapper {
        self.slab.get(*index)
    }

    pub(crate) fn replace_decl_id(&self, index: DeclId, wrapper: DeclWrapper) {
        self.slab.replace(index, wrapper);
    }

    pub(crate) fn insert(&self, declaration_wrapper: DeclWrapper, span: Span) -> DeclId {
        DeclId::new(self.slab.insert(declaration_wrapper), span)
    }

    /// Given a [DeclarationId] `index`, finds all the parents of `index` and
    /// all the recursive parents of those parents, and so on. Does not perform
    /// duplicated computation---if the parents of a [DeclarationId] have
    /// already been found, we do not find them again.
    #[allow(clippy::map_entry)]
    pub(crate) fn find_all_parents(&self, engines: Engines<'_>, index: DeclId) -> Vec<DeclId> {
        let parents = self.parents.read().unwrap();
        let mut acc_parents: HashMap<usize, DeclId> = HashMap::new();
        let mut already_checked: HashSet<usize> = HashSet::new();
        let mut left_to_check: VecDeque<DeclId> = VecDeque::from([index]);
        while let Some(curr) = left_to_check.pop_front() {
            if !already_checked.insert(*curr) {
                continue;
            }
            if let Some(curr_parents) = parents.get(&*curr) {
                for curr_parent in curr_parents.iter() {
                    if !acc_parents.contains_key(&**curr_parent) {
                        acc_parents.insert(**curr_parent, curr_parent.clone());
                    }
                    if !left_to_check.iter().any(|x| x.eq(curr_parent, engines)) {
                        left_to_check.push_back(curr_parent.clone());
                    }
                }
            }
        }
        acc_parents.values().cloned().collect()
    }

    pub(crate) fn register_parent(&self, index: &DeclId, parent: DeclId) {
        let mut parents = self.parents.write().unwrap();
        parents
            .entry(**index)
            .and_modify(|e| e.push(parent.clone()))
            .or_insert_with(|| vec![parent]);
    }

    pub(crate) fn insert_function(&self, function: ty::TyFunctionDeclaration) -> DeclId {
        let span = function.span();
        self.insert(DeclWrapper::Function(function), span)
    }

    pub fn get_function(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyFunctionDeclaration, CompileError> {
        self.slab.get(*index).expect_function(span)
    }

    pub(crate) fn insert_trait(&self, r#trait: ty::TyTraitDeclaration) -> DeclId {
        let span = r#trait.name.span();
        self.insert(DeclWrapper::Trait(r#trait), span)
    }

    pub fn get_trait(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyTraitDeclaration, CompileError> {
        self.slab.get(*index).expect_trait(span)
    }

    pub(crate) fn insert_trait_fn(&self, trait_fn: ty::TyTraitFn) -> DeclId {
        let span = trait_fn.name.span();
        self.insert(DeclWrapper::TraitFn(trait_fn), span)
    }

    pub fn get_trait_fn(&self, index: DeclId, span: &Span) -> Result<ty::TyTraitFn, CompileError> {
        self.slab.get(*index).expect_trait_fn(span)
    }

    pub(crate) fn insert_impl_trait(&self, impl_trait: ty::TyImplTrait) -> DeclId {
        let span = impl_trait.span.clone();
        self.insert(DeclWrapper::ImplTrait(impl_trait), span)
    }

    pub fn get_impl_trait(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyImplTrait, CompileError> {
        self.slab.get(*index).expect_impl_trait(span)
    }

    pub(crate) fn insert_struct(&self, r#struct: ty::TyStructDeclaration) -> DeclId {
        let span = r#struct.span();
        self.insert(DeclWrapper::Struct(r#struct), span)
    }

    pub fn get_struct(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyStructDeclaration, CompileError> {
        self.slab.get(*index).expect_struct(span)
    }

    pub(crate) fn insert_storage(&self, storage: ty::TyStorageDeclaration) -> DeclId {
        let span = storage.span();
        self.insert(DeclWrapper::Storage(storage), span)
    }

    pub fn get_storage(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyStorageDeclaration, CompileError> {
        self.slab.get(*index).expect_storage(span)
    }

    pub(crate) fn insert_abi(&self, abi: ty::TyAbiDeclaration) -> DeclId {
        let span = abi.span.clone();
        self.insert(DeclWrapper::Abi(abi), span)
    }

    pub fn get_abi(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyAbiDeclaration, CompileError> {
        self.slab.get(*index).expect_abi(span)
    }

    pub(crate) fn insert_constant(&self, constant: ty::TyConstantDeclaration) -> DeclId {
        let span = constant.name.span();
        DeclId::new(
            self.slab.insert(DeclWrapper::Constant(Box::new(constant))),
            span,
        )
    }

    pub fn get_constant(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyConstantDeclaration, CompileError> {
        self.slab.get(*index).expect_constant(span)
    }

    pub(crate) fn insert_enum(&self, enum_decl: ty::TyEnumDeclaration) -> DeclId {
        let span = enum_decl.span();
        self.insert(DeclWrapper::Enum(enum_decl), span)
    }

    pub fn get_enum(
        &self,
        index: DeclId,
        span: &Span,
    ) -> Result<ty::TyEnumDeclaration, CompileError> {
        self.slab.get(*index).expect_enum(span)
    }
}
