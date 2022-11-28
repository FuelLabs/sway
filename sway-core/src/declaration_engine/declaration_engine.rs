use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    sync::RwLock,
};

use lazy_static::lazy_static;
use sway_error::error::CompileError;
use sway_types::{Span, Spanned};

use crate::{
    concurrent_slab::{ConcurrentSlab, ListDisplay},
    language::ty,
    PartialEqWithTypeEngine, TypeEngine,
};

use super::{declaration_id::DeclarationId, declaration_wrapper::DeclarationWrapper};

lazy_static! {
    static ref DECLARATION_ENGINE: DeclarationEngine = DeclarationEngine::default();
}

/// Used inside of type inference to store declarations.
#[derive(Debug, Default)]
pub struct DeclarationEngine {
    slab: ConcurrentSlab<DeclarationWrapper>,
    parents: RwLock<HashMap<usize, Vec<DeclarationId>>>,
}

impl fmt::Display for DeclarationEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.slab.with_slice(|elems| {
            let list = ListDisplay { list: elems.iter() };
            write!(f, "DeclarationEngine {{\n{}\n}}", list)
        })
    }
}

impl DeclarationEngine {
    fn clear(&self) {
        self.slab.clear();
        let mut parents = self.parents.write().unwrap();
        parents.clear();
    }

    fn look_up_decl_id(&self, index: DeclarationId) -> DeclarationWrapper {
        self.slab.get(*index)
    }

    fn replace_decl_id(&self, index: DeclarationId, wrapper: DeclarationWrapper) {
        self.slab.replace(index, wrapper);
    }

    fn insert(&self, declaration_wrapper: DeclarationWrapper, span: Span) -> DeclarationId {
        DeclarationId::new(self.slab.insert(declaration_wrapper), span)
    }

    /// Given a [DeclarationId] `index`, finds all the parents of `index` and
    /// all the recursive parents of those parents, and so on. Does not perform
    /// duplicated computation---if the parents of a [DeclarationId] have
    /// already been found, we do not find them again.
    #[allow(clippy::map_entry)]
    fn find_all_parents(
        &self,
        index: DeclarationId,
        type_engine: &TypeEngine,
    ) -> Vec<DeclarationId> {
        let parents = self.parents.read().unwrap();
        let mut acc_parents: HashMap<usize, DeclarationId> = HashMap::new();
        let mut already_checked: HashSet<usize> = HashSet::new();
        let mut left_to_check: VecDeque<DeclarationId> = VecDeque::from([index]);
        while let Some(curr) = left_to_check.pop_front() {
            if !already_checked.insert(*curr) {
                continue;
            }
            if let Some(curr_parents) = parents.get(&*curr) {
                for curr_parent in curr_parents.iter() {
                    if !acc_parents.contains_key(&**curr_parent) {
                        acc_parents.insert(**curr_parent, curr_parent.clone());
                    }
                    if !left_to_check.iter().any(|x| x.eq(curr_parent, type_engine)) {
                        left_to_check.push_back(curr_parent.clone());
                    }
                }
            }
        }
        acc_parents.values().cloned().collect()
    }

    fn register_parent(&self, index: &DeclarationId, parent: DeclarationId) {
        let mut parents = self.parents.write().unwrap();
        parents
            .entry(**index)
            .and_modify(|e| e.push(parent.clone()))
            .or_insert_with(|| vec![parent]);
    }

    fn insert_function(&self, function: ty::TyFunctionDeclaration) -> DeclarationId {
        let span = function.span();
        self.insert(DeclarationWrapper::Function(function), span)
    }

    fn get_function(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<ty::TyFunctionDeclaration, CompileError> {
        self.slab.get(*index).expect_function(span)
    }

    fn insert_trait(&self, r#trait: ty::TyTraitDeclaration) -> DeclarationId {
        let span = r#trait.name.span();
        self.insert(DeclarationWrapper::Trait(r#trait), span)
    }

    fn get_trait(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<ty::TyTraitDeclaration, CompileError> {
        self.slab.get(*index).expect_trait(span)
    }

    fn insert_trait_fn(&self, trait_fn: ty::TyTraitFn) -> DeclarationId {
        let span = trait_fn.name.span();
        self.insert(DeclarationWrapper::TraitFn(trait_fn), span)
    }

    fn get_trait_fn(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<ty::TyTraitFn, CompileError> {
        self.slab.get(*index).expect_trait_fn(span)
    }

    fn insert_impl_trait(&self, impl_trait: ty::TyImplTrait) -> DeclarationId {
        let span = impl_trait.span.clone();
        self.insert(DeclarationWrapper::ImplTrait(impl_trait), span)
    }

    fn get_impl_trait(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<ty::TyImplTrait, CompileError> {
        self.slab.get(*index).expect_impl_trait(span)
    }

    fn insert_struct(&self, r#struct: ty::TyStructDeclaration) -> DeclarationId {
        let span = r#struct.span();
        self.insert(DeclarationWrapper::Struct(r#struct), span)
    }

    fn get_struct(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<ty::TyStructDeclaration, CompileError> {
        self.slab.get(*index).expect_struct(span)
    }

    fn insert_storage(&self, storage: ty::TyStorageDeclaration) -> DeclarationId {
        let span = storage.span();
        self.insert(DeclarationWrapper::Storage(storage), span)
    }

    fn get_storage(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<ty::TyStorageDeclaration, CompileError> {
        self.slab.get(*index).expect_storage(span)
    }

    fn insert_abi(&self, abi: ty::TyAbiDeclaration) -> DeclarationId {
        let span = abi.span.clone();
        self.insert(DeclarationWrapper::Abi(abi), span)
    }

    fn get_abi(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<ty::TyAbiDeclaration, CompileError> {
        self.slab.get(*index).expect_abi(span)
    }

    fn insert_constant(&self, constant: ty::TyConstantDeclaration) -> DeclarationId {
        let span = constant.name.span();
        DeclarationId::new(
            self.slab
                .insert(DeclarationWrapper::Constant(Box::new(constant))),
            span,
        )
    }

    fn get_constant(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<ty::TyConstantDeclaration, CompileError> {
        self.slab.get(*index).expect_constant(span)
    }

    fn insert_enum(&self, enum_decl: ty::TyEnumDeclaration) -> DeclarationId {
        let span = enum_decl.span();
        self.insert(DeclarationWrapper::Enum(enum_decl), span)
    }

    fn get_enum(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<ty::TyEnumDeclaration, CompileError> {
        self.slab.get(*index).expect_enum(span)
    }
}

#[allow(dead_code)]
pub(crate) fn de_print() {
    println!("{}", &*DECLARATION_ENGINE);
}

pub(crate) fn de_clear() {
    DECLARATION_ENGINE.clear();
}

pub fn de_look_up_decl_id(index: DeclarationId) -> DeclarationWrapper {
    DECLARATION_ENGINE.look_up_decl_id(index)
}

pub(crate) fn de_replace_decl_id(index: DeclarationId, wrapper: DeclarationWrapper) {
    DECLARATION_ENGINE.replace_decl_id(index, wrapper)
}

pub(crate) fn de_insert(declaration_wrapper: DeclarationWrapper, span: Span) -> DeclarationId {
    DECLARATION_ENGINE.insert(declaration_wrapper, span)
}

pub(super) fn de_find_all_parents(
    index: DeclarationId,
    type_engine: &TypeEngine,
) -> Vec<DeclarationId> {
    DECLARATION_ENGINE.find_all_parents(index, type_engine)
}

pub(super) fn de_register_parent(index: &DeclarationId, parent: DeclarationId) {
    DECLARATION_ENGINE.register_parent(index, parent);
}

pub(crate) fn de_insert_function(function: ty::TyFunctionDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_function(function)
}

pub fn de_get_function(
    index: DeclarationId,
    span: &Span,
) -> Result<ty::TyFunctionDeclaration, CompileError> {
    DECLARATION_ENGINE.get_function(index, span)
}

pub(crate) fn de_insert_trait(r#trait: ty::TyTraitDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_trait(r#trait)
}

pub fn de_get_trait(
    index: DeclarationId,
    span: &Span,
) -> Result<ty::TyTraitDeclaration, CompileError> {
    DECLARATION_ENGINE.get_trait(index, span)
}

pub(crate) fn de_insert_trait_fn(trait_fn: ty::TyTraitFn) -> DeclarationId {
    DECLARATION_ENGINE.insert_trait_fn(trait_fn)
}

pub fn de_get_trait_fn(index: DeclarationId, span: &Span) -> Result<ty::TyTraitFn, CompileError> {
    DECLARATION_ENGINE.get_trait_fn(index, span)
}

pub(crate) fn de_insert_impl_trait(impl_trait: ty::TyImplTrait) -> DeclarationId {
    DECLARATION_ENGINE.insert_impl_trait(impl_trait)
}

pub fn de_get_impl_trait(
    index: DeclarationId,
    span: &Span,
) -> Result<ty::TyImplTrait, CompileError> {
    DECLARATION_ENGINE.get_impl_trait(index, span)
}

pub(crate) fn de_insert_struct(r#struct: ty::TyStructDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_struct(r#struct)
}

pub fn de_get_struct(
    index: DeclarationId,
    span: &Span,
) -> Result<ty::TyStructDeclaration, CompileError> {
    DECLARATION_ENGINE.get_struct(index, span)
}

pub(crate) fn de_insert_storage(storage: ty::TyStorageDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_storage(storage)
}

pub fn de_get_storage(
    index: DeclarationId,
    span: &Span,
) -> Result<ty::TyStorageDeclaration, CompileError> {
    DECLARATION_ENGINE.get_storage(index, span)
}

pub(crate) fn de_insert_abi(abi: ty::TyAbiDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_abi(abi)
}

pub fn de_get_abi(index: DeclarationId, span: &Span) -> Result<ty::TyAbiDeclaration, CompileError> {
    DECLARATION_ENGINE.get_abi(index, span)
}

pub(crate) fn de_insert_constant(constant: ty::TyConstantDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_constant(constant)
}

pub fn de_get_constant(
    index: DeclarationId,
    span: &Span,
) -> Result<ty::TyConstantDeclaration, CompileError> {
    DECLARATION_ENGINE.get_constant(index, span)
}

pub(crate) fn de_insert_enum(enum_decl: ty::TyEnumDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_enum(enum_decl)
}

pub fn de_get_enum(
    index: DeclarationId,
    span: &Span,
) -> Result<ty::TyEnumDeclaration, CompileError> {
    DECLARATION_ENGINE.get_enum(index, span)
}
