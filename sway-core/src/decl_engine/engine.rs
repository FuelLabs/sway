use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    sync::RwLock,
};

use sway_error::error::CompileError;
use sway_types::{Ident, Span};

use crate::{
    concurrent_slab::{ConcurrentSlab, ListDisplay},
    decl_engine::*,
    engine_threading::*,
    language::ty,
};

/// Used inside of type inference to store declarations.
#[derive(Debug, Default)]
pub struct DeclEngine {
    slab: ConcurrentSlab<DeclWrapper>,
    parents: RwLock<HashMap<DeclId, Vec<DeclRef>>>,
}

impl fmt::Display for DeclEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.slab.with_slice(|elems| {
            let list = ListDisplay { list: elems.iter() };
            write!(f, "DeclarationEngine {{\n{list}\n}}")
        })
    }
}

impl DeclEngine {
    pub(crate) fn get(&self, index: &DeclRef) -> DeclWrapper {
        self.slab.get(*DeclId::from(index))
    }

    pub(super) fn replace(&self, index: &DeclRef, wrapper: DeclWrapper) {
        self.slab.replace(DeclId::from(index), wrapper);
    }

    pub(crate) fn insert<T>(&self, decl: T) -> DeclRef
    where
        T: Into<(Ident, DeclWrapper, Span)>,
    {
        let (ident, decl_wrapper, span) = decl.into();
        DeclRef::new(ident, self.slab.insert(decl_wrapper), span)
    }

    pub(crate) fn insert_wrapper(
        &self,
        ident: Ident,
        decl_wrapper: DeclWrapper,
        span: Span,
    ) -> DeclRef {
        DeclRef::new(ident, self.slab.insert(decl_wrapper), span)
    }

    /// Given a [DeclId] `index`, finds all the parents of `index` and all the
    /// recursive parents of those parents, and so on. Does not perform
    /// duplicated computation---if the parents of a [DeclId] have already been
    /// found, we do not find them again.
    #[allow(clippy::map_entry)]
    pub(crate) fn find_all_parents(&self, engines: Engines<'_>, index: &DeclRef) -> Vec<DeclRef> {
        let parents = self.parents.read().unwrap();
        let mut acc_parents: HashMap<DeclId, &DeclRef> = HashMap::new();
        let mut already_checked: HashSet<DeclId> = HashSet::new();
        let mut left_to_check: VecDeque<&DeclRef> = VecDeque::from([index]);
        while let Some(curr) = left_to_check.pop_front() {
            if !already_checked.insert(DeclId::from(curr)) {
                continue;
            }
            if let Some(curr_parents) = parents.get(&DeclId::from(curr)) {
                for curr_parent in curr_parents.iter() {
                    if !acc_parents.contains_key(&DeclId::from(curr_parent)) {
                        acc_parents.insert(DeclId::from(curr_parent), curr_parent);
                    }
                    if !left_to_check.iter().any(|x| x.eq(&curr_parent, engines)) {
                        left_to_check.push_back(curr_parent);
                    }
                }
            }
        }
        acc_parents.values().cloned().cloned().collect()
    }

    pub(crate) fn register_parent(&self, index: &DeclRef, parent: DeclRef) {
        let mut parents = self.parents.write().unwrap();
        parents
            .entry(DeclId::from(index))
            .and_modify(|e| e.push(parent.clone()))
            .or_insert_with(|| vec![parent]);
    }

    pub fn get_function(
        &self,
        index: &DeclRef,
        span: &Span,
    ) -> Result<ty::TyFunctionDeclaration, CompileError> {
        self.slab.get(*DeclId::from(index)).expect_function(span)
    }

    pub fn get_trait(
        &self,
        index: &DeclRef,
        span: &Span,
    ) -> Result<ty::TyTraitDeclaration, CompileError> {
        self.slab.get(*DeclId::from(index)).expect_trait(span)
    }

    pub fn get_trait_fn(
        &self,
        index: &DeclRef,
        span: &Span,
    ) -> Result<ty::TyTraitFn, CompileError> {
        self.slab.get(*DeclId::from(index)).expect_trait_fn(span)
    }

    pub fn get_impl_trait(
        &self,
        index: &DeclRef,
        span: &Span,
    ) -> Result<ty::TyImplTrait, CompileError> {
        self.slab.get(*DeclId::from(index)).expect_impl_trait(span)
    }

    pub fn get_struct(
        &self,
        index: &DeclRef,
        span: &Span,
    ) -> Result<ty::TyStructDeclaration, CompileError> {
        self.slab.get(*DeclId::from(index)).expect_struct(span)
    }

    pub fn get_storage(
        &self,
        index: &DeclRef,
        span: &Span,
    ) -> Result<ty::TyStorageDeclaration, CompileError> {
        self.slab.get(*DeclId::from(index)).expect_storage(span)
    }

    pub fn get_abi(
        &self,
        index: &DeclRef,
        span: &Span,
    ) -> Result<ty::TyAbiDeclaration, CompileError> {
        self.slab.get(*DeclId::from(index)).expect_abi(span)
    }

    pub fn get_constant(
        &self,
        index: &DeclRef,
        span: &Span,
    ) -> Result<ty::TyConstantDeclaration, CompileError> {
        self.slab.get(*DeclId::from(index)).expect_constant(span)
    }

    pub fn get_enum(
        &self,
        index: &DeclRef,
        span: &Span,
    ) -> Result<ty::TyEnumDeclaration, CompileError> {
        self.slab.get(*DeclId::from(index)).expect_enum(span)
    }
}
