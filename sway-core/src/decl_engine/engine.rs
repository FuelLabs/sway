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
    parents: RwLock<HashMap<DeclId, Vec<DeclId>>>,
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
    pub(crate) fn get<'a, T>(&self, index: &'a T) -> DeclWrapper
    where
        DeclId: From<&'a T>,
    {
        self.slab.get(*DeclId::from(index))
    }

    /// This method was added as a weird workaround for a potential Rust
    /// compiler bug where it is unable to resolve the trait constraints on the
    /// `get` method above (???)
    fn get_from_decl_id(&self, index: DeclId) -> DeclWrapper {
        self.slab.get(*index)
    }

    pub(super) fn replace<'a, T>(&self, index: &'a T, wrapper: DeclWrapper)
    where
        DeclId: From<&'a T>,
    {
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

    /// Given a [DeclRef] `index`, finds all the parents of `index` and all the
    /// recursive parents of those parents, and so on. Does not perform
    /// duplicated computation---if the parents of a [DeclRef] have already been
    /// found, we do not find them again.
    #[allow(clippy::map_entry)]
    pub(crate) fn find_all_parents<'a, T>(&self, engines: Engines<'_>, index: &'a T) -> Vec<DeclId>
    where
        DeclId: From<&'a T>,
    {
        let index: DeclId = DeclId::from(index);
        let parents = self.parents.read().unwrap();
        let mut acc_parents: HashMap<DeclId, DeclId> = HashMap::new();
        let mut already_checked: HashSet<DeclId> = HashSet::new();
        let mut left_to_check: VecDeque<DeclId> = VecDeque::from([index]);
        while let Some(curr) = left_to_check.pop_front() {
            if !already_checked.insert(curr) {
                continue;
            }
            if let Some(curr_parents) = parents.get(&curr) {
                for curr_parent in curr_parents.iter() {
                    if !acc_parents.contains_key(curr_parent) {
                        acc_parents.insert(*curr_parent, *curr_parent);
                    }
                    if !left_to_check.iter().any(|x| {
                        self.get_from_decl_id(*x)
                            .eq(&self.get_from_decl_id(*curr_parent), engines)
                    }) {
                        left_to_check.push_back(*curr_parent);
                    }
                }
            }
        }
        acc_parents.values().cloned().collect()
    }

    pub(crate) fn register_parent<'a, T>(&self, index: &DeclRef, parent: &'a T)
    where
        DeclId: From<&'a T>,
    {
        let index: DeclId = index.into();
        let parent: DeclId = DeclId::from(parent);
        let mut parents = self.parents.write().unwrap();
        parents
            .entry(index)
            .and_modify(|e| e.push(parent))
            .or_insert_with(|| vec![parent]);
    }

    pub fn get_function<'a, T>(
        &self,
        index: &'a T,
        span: &Span,
    ) -> Result<ty::TyFunctionDeclaration, CompileError>
    where
        DeclId: From<&'a T>,
    {
        self.slab.get(*DeclId::from(index)).expect_function(span)
    }

    pub fn get_trait<'a, T>(
        &self,
        index: &'a T,
        span: &Span,
    ) -> Result<ty::TyTraitDeclaration, CompileError>
    where
        DeclId: From<&'a T>,
    {
        self.slab.get(*DeclId::from(index)).expect_trait(span)
    }

    pub fn get_trait_fn<'a, T>(
        &self,
        index: &'a T,
        span: &Span,
    ) -> Result<ty::TyTraitFn, CompileError>
    where
        DeclId: From<&'a T>,
    {
        self.slab.get(*DeclId::from(index)).expect_trait_fn(span)
    }

    pub fn get_impl_trait<'a, T>(
        &self,
        index: &'a T,
        span: &Span,
    ) -> Result<ty::TyImplTrait, CompileError>
    where
        DeclId: From<&'a T>,
    {
        self.slab.get(*DeclId::from(index)).expect_impl_trait(span)
    }

    pub fn get_struct<'a, T>(
        &self,
        index: &'a T,
        span: &Span,
    ) -> Result<ty::TyStructDeclaration, CompileError>
    where
        DeclId: From<&'a T>,
    {
        self.slab.get(*DeclId::from(index)).expect_struct(span)
    }

    pub fn get_storage<'a, T>(
        &self,
        index: &'a T,
        span: &Span,
    ) -> Result<ty::TyStorageDeclaration, CompileError>
    where
        DeclId: From<&'a T>,
    {
        self.slab.get(*DeclId::from(index)).expect_storage(span)
    }

    pub fn get_abi<'a, T>(
        &self,
        index: &'a T,
        span: &Span,
    ) -> Result<ty::TyAbiDeclaration, CompileError>
    where
        DeclId: From<&'a T>,
    {
        self.slab.get(*DeclId::from(index)).expect_abi(span)
    }

    pub fn get_constant<'a, T>(
        &self,
        index: &'a T,
        span: &Span,
    ) -> Result<ty::TyConstantDeclaration, CompileError>
    where
        DeclId: From<&'a T>,
    {
        self.slab.get(*DeclId::from(index)).expect_constant(span)
    }

    pub fn get_enum<'a, T>(
        &self,
        index: &'a T,
        span: &Span,
    ) -> Result<ty::TyEnumDeclaration, CompileError>
    where
        DeclId: From<&'a T>,
    {
        self.slab.get(*DeclId::from(index)).expect_enum(span)
    }
}
