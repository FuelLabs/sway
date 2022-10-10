use lazy_static::lazy_static;
use std::{collections::HashMap, sync::RwLock};
use sway_error::error::CompileError;
use sway_types::{Span, Spanned};

use crate::{
    concurrent_slab::ConcurrentSlab,
    semantic_analysis::{
        TyAbiDeclaration, TyConstantDeclaration, TyEnumDeclaration, TyImplTrait,
        TyStorageDeclaration, TyStructDeclaration, TyTraitDeclaration, TyTraitFn,
    },
    TyFunctionDeclaration,
};

use super::{declaration_id::DeclarationId, declaration_wrapper::DeclarationWrapper};

lazy_static! {
    static ref DECLARATION_ENGINE: DeclarationEngine = DeclarationEngine::default();
}

/// Used inside of type inference to store declarations.
#[derive(Debug, Default)]
pub(crate) struct DeclarationEngine {
    slab: ConcurrentSlab<DeclarationWrapper>,
    // *declaration_id -> vec of monomorphized copies
    // where the declaration_id is the original declaration
    monomorphized_copies: RwLock<HashMap<usize, Vec<DeclarationId>>>,
}

impl DeclarationEngine {
    fn clear(&self) {
        self.slab.clear();
        let mut monomorphized_copies = self.monomorphized_copies.write().unwrap();
        monomorphized_copies.clear();
    }

    fn look_up_decl_id(&self, index: DeclarationId) -> DeclarationWrapper {
        self.slab.get(*index)
    }

    fn replace_decl_id(&self, index: DeclarationId, wrapper: DeclarationWrapper) {
        self.slab.replace(index, wrapper);
    }

    fn add_monomorphized_copy(&self, original_id: DeclarationId, new_id: DeclarationId) {
        let mut monomorphized_copies = self.monomorphized_copies.write().unwrap();
        match monomorphized_copies.get_mut(&*original_id) {
            Some(prev) => {
                prev.push(new_id);
            }
            None => {
                monomorphized_copies.insert(*original_id, vec![new_id]);
            }
        }
    }

    fn get_monomorphized_copies(&self, original_id: DeclarationId) -> Vec<DeclarationWrapper> {
        let monomorphized_copies = self.monomorphized_copies.write().unwrap();
        match monomorphized_copies.get(&*original_id).cloned() {
            Some(copies) => copies
                .into_iter()
                .map(|copy| self.slab.get(*copy))
                .collect(),
            None => vec![],
        }
    }

    fn insert_function(&self, function: TyFunctionDeclaration) -> DeclarationId {
        let span = function.span();
        DeclarationId::new(
            self.slab.insert(DeclarationWrapper::Function(function)),
            span,
        )
    }

    fn get_function(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TyFunctionDeclaration, CompileError> {
        self.slab.get(*index).expect_function(span)
    }

    fn add_monomorphized_function_copy(
        &self,
        original_id: DeclarationId,
        new_copy: TyFunctionDeclaration,
    ) {
        let span = new_copy.span();
        let new_id = DeclarationId::new(
            self.slab.insert(DeclarationWrapper::Function(new_copy)),
            span,
        );
        self.add_monomorphized_copy(original_id, new_id)
    }

    fn get_monomorphized_function_copies(
        &self,
        original_id: DeclarationId,
        span: &Span,
    ) -> Result<Vec<TyFunctionDeclaration>, CompileError> {
        self.get_monomorphized_copies(original_id)
            .into_iter()
            .map(|x| x.expect_function(span))
            .collect::<Result<_, _>>()
    }

    fn insert_trait(&self, r#trait: TyTraitDeclaration) -> DeclarationId {
        let span = r#trait.name.span();
        DeclarationId::new(self.slab.insert(DeclarationWrapper::Trait(r#trait)), span)
    }

    fn get_trait(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TyTraitDeclaration, CompileError> {
        self.slab.get(*index).expect_trait(span)
    }

    fn insert_trait_fn(&self, trait_fn: TyTraitFn) -> DeclarationId {
        let span = trait_fn.name.span();
        DeclarationId::new(
            self.slab.insert(DeclarationWrapper::TraitFn(trait_fn)),
            span,
        )
    }

    fn get_trait_fn(&self, index: DeclarationId, span: &Span) -> Result<TyTraitFn, CompileError> {
        self.slab.get(*index).expect_trait_fn(span)
    }

    fn insert_impl_trait(&self, impl_trait: TyImplTrait) -> DeclarationId {
        let span = impl_trait.span.clone();
        DeclarationId::new(
            self.slab.insert(DeclarationWrapper::ImplTrait(impl_trait)),
            span,
        )
    }

    fn get_impl_trait(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TyImplTrait, CompileError> {
        self.slab.get(*index).expect_impl_trait(span)
    }

    fn insert_struct(&self, r#struct: TyStructDeclaration) -> DeclarationId {
        let span = r#struct.span();
        DeclarationId::new(self.slab.insert(DeclarationWrapper::Struct(r#struct)), span)
    }

    fn get_struct(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TyStructDeclaration, CompileError> {
        self.slab.get(*index).expect_struct(span)
    }

    fn add_monomorphized_struct_copy(
        &self,
        original_id: DeclarationId,
        new_copy: TyStructDeclaration,
    ) {
        let span = new_copy.span();
        let new_id =
            DeclarationId::new(self.slab.insert(DeclarationWrapper::Struct(new_copy)), span);
        self.add_monomorphized_copy(original_id, new_id)
    }

    fn get_monomorphized_struct_copies(
        &self,
        original_id: DeclarationId,
        span: &Span,
    ) -> Result<Vec<TyStructDeclaration>, CompileError> {
        self.get_monomorphized_copies(original_id)
            .into_iter()
            .map(|x| x.expect_struct(span))
            .collect::<Result<_, _>>()
    }

    fn insert_storage(&self, storage: TyStorageDeclaration) -> DeclarationId {
        let span = storage.span();
        DeclarationId::new(self.slab.insert(DeclarationWrapper::Storage(storage)), span)
    }

    fn get_storage(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TyStorageDeclaration, CompileError> {
        self.slab.get(*index).expect_storage(span)
    }

    fn insert_abi(&self, abi: TyAbiDeclaration) -> DeclarationId {
        let span = abi.span.clone();
        DeclarationId::new(self.slab.insert(DeclarationWrapper::Abi(abi)), span)
    }

    fn get_abi(&self, index: DeclarationId, span: &Span) -> Result<TyAbiDeclaration, CompileError> {
        self.slab.get(*index).expect_abi(span)
    }

    fn insert_constant(&self, constant: TyConstantDeclaration) -> DeclarationId {
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
    ) -> Result<TyConstantDeclaration, CompileError> {
        self.slab.get(*index).expect_constant(span)
    }

    fn insert_enum(&self, enum_decl: TyEnumDeclaration) -> DeclarationId {
        let span = enum_decl.span();
        DeclarationId::new(self.slab.insert(DeclarationWrapper::Enum(enum_decl)), span)
    }

    fn get_enum(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TyEnumDeclaration, CompileError> {
        self.slab.get(*index).expect_enum(span)
    }

    fn add_monomorphized_enum_copy(&self, original_id: DeclarationId, new_copy: TyEnumDeclaration) {
        let span = new_copy.span();
        let new_id = DeclarationId::new(self.slab.insert(DeclarationWrapper::Enum(new_copy)), span);
        self.add_monomorphized_copy(original_id, new_id)
    }
}

pub(crate) fn de_clear() {
    DECLARATION_ENGINE.clear()
}

pub(crate) fn de_look_up_decl_id(index: DeclarationId) -> DeclarationWrapper {
    DECLARATION_ENGINE.look_up_decl_id(index)
}

pub(crate) fn de_replace_decl_id(index: DeclarationId, wrapper: DeclarationWrapper) {
    DECLARATION_ENGINE.replace_decl_id(index, wrapper)
}

pub(crate) fn de_add_monomorphized_copy(original_id: DeclarationId, new_id: DeclarationId) {
    DECLARATION_ENGINE.add_monomorphized_copy(original_id, new_id);
}

pub(crate) fn de_insert_function(function: TyFunctionDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_function(function)
}

pub fn de_get_function(
    index: DeclarationId,
    span: &Span,
) -> Result<TyFunctionDeclaration, CompileError> {
    DECLARATION_ENGINE.get_function(index, span)
}

pub(crate) fn de_add_monomorphized_function_copy(
    original_id: DeclarationId,
    new_copy: TyFunctionDeclaration,
) {
    DECLARATION_ENGINE.add_monomorphized_function_copy(original_id, new_copy);
}

pub(crate) fn de_get_monomorphized_function_copies(
    original_id: DeclarationId,
    span: &Span,
) -> Result<Vec<TyFunctionDeclaration>, CompileError> {
    DECLARATION_ENGINE.get_monomorphized_function_copies(original_id, span)
}

pub(crate) fn de_insert_trait(r#trait: TyTraitDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_trait(r#trait)
}

pub fn de_get_trait(index: DeclarationId, span: &Span) -> Result<TyTraitDeclaration, CompileError> {
    DECLARATION_ENGINE.get_trait(index, span)
}

pub(crate) fn de_insert_trait_fn(trait_fn: TyTraitFn) -> DeclarationId {
    DECLARATION_ENGINE.insert_trait_fn(trait_fn)
}

pub(crate) fn de_get_trait_fn(
    index: DeclarationId,
    span: &Span,
) -> Result<TyTraitFn, CompileError> {
    DECLARATION_ENGINE.get_trait_fn(index, span)
}

pub(crate) fn de_insert_impl_trait(impl_trait: TyImplTrait) -> DeclarationId {
    DECLARATION_ENGINE.insert_impl_trait(impl_trait)
}

pub fn de_get_impl_trait(index: DeclarationId, span: &Span) -> Result<TyImplTrait, CompileError> {
    DECLARATION_ENGINE.get_impl_trait(index, span)
}

pub(crate) fn de_insert_struct(r#struct: TyStructDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_struct(r#struct)
}

pub fn de_get_struct(
    index: DeclarationId,
    span: &Span,
) -> Result<TyStructDeclaration, CompileError> {
    DECLARATION_ENGINE.get_struct(index, span)
}

pub(crate) fn de_add_monomorphized_struct_copy(
    original_id: DeclarationId,
    new_copy: TyStructDeclaration,
) {
    DECLARATION_ENGINE.add_monomorphized_struct_copy(original_id, new_copy);
}

pub(crate) fn de_get_monomorphized_struct_copies(
    original_id: DeclarationId,
    span: &Span,
) -> Result<Vec<TyStructDeclaration>, CompileError> {
    DECLARATION_ENGINE.get_monomorphized_struct_copies(original_id, span)
}

pub(crate) fn de_insert_storage(storage: TyStorageDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_storage(storage)
}

pub fn de_get_storage(
    index: DeclarationId,
    span: &Span,
) -> Result<TyStorageDeclaration, CompileError> {
    DECLARATION_ENGINE.get_storage(index, span)
}

pub(crate) fn de_insert_abi(abi: TyAbiDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_abi(abi)
}

pub fn de_get_abi(index: DeclarationId, span: &Span) -> Result<TyAbiDeclaration, CompileError> {
    DECLARATION_ENGINE.get_abi(index, span)
}

pub(crate) fn de_insert_constant(constant: TyConstantDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_constant(constant)
}

pub fn de_get_constant(
    index: DeclarationId,
    span: &Span,
) -> Result<TyConstantDeclaration, CompileError> {
    DECLARATION_ENGINE.get_constant(index, span)
}

pub(crate) fn de_insert_enum(enum_decl: TyEnumDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.insert_enum(enum_decl)
}

pub fn de_get_enum(index: DeclarationId, span: &Span) -> Result<TyEnumDeclaration, CompileError> {
    DECLARATION_ENGINE.get_enum(index, span)
}

pub(crate) fn de_add_monomorphized_enum_copy(
    original_id: DeclarationId,
    new_copy: TyEnumDeclaration,
) {
    DECLARATION_ENGINE.add_monomorphized_enum_copy(original_id, new_copy);
}
