use lazy_static::lazy_static;
use std::{collections::HashMap, sync::RwLock};
use sway_types::{Span, Spanned};

use crate::{
    concurrent_slab::ConcurrentSlab,
    semantic_analysis::{
        TypedImplTrait, TypedStructDeclaration, TypedTraitDeclaration, TypedTraitFn,
    },
    CompileError, TypedFunctionDeclaration,
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

    fn de_look_up_decl_id(&self, index: DeclarationId) -> DeclarationWrapper {
        self.slab.get(*index)
    }

    fn de_add_monomorphized_copy(&self, original_id: DeclarationId, new_id: DeclarationId) {
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

    fn de_get_monomorphized_copies(&self, original_id: DeclarationId) -> Vec<DeclarationWrapper> {
        let monomorphized_copies = self.monomorphized_copies.write().unwrap();
        match monomorphized_copies.get(&*original_id).cloned() {
            Some(copies) => copies
                .into_iter()
                .map(|copy| self.slab.get(*copy))
                .collect(),
            None => vec![],
        }
    }

    fn de_insert_function(&self, function: TypedFunctionDeclaration) -> DeclarationId {
        let span = function.span();
        DeclarationId::new(
            self.slab.insert(DeclarationWrapper::Function(function)),
            span,
        )
    }

    fn de_get_function(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TypedFunctionDeclaration, CompileError> {
        self.slab.get(*index).expect_function(span)
    }

    fn de_add_monomorphized_function_copy(
        &self,
        original_id: DeclarationId,
        new_copy: TypedFunctionDeclaration,
    ) {
        let span = new_copy.span();
        let new_id = DeclarationId::new(
            self.slab.insert(DeclarationWrapper::Function(new_copy)),
            span,
        );
        self.de_add_monomorphized_copy(original_id, new_id)
    }

    fn de_get_monomorphized_function_copies(
        &self,
        original_id: DeclarationId,
        span: &Span,
    ) -> Result<Vec<TypedFunctionDeclaration>, CompileError> {
        self.de_get_monomorphized_copies(original_id)
            .into_iter()
            .map(|x| x.expect_function(span))
            .collect::<Result<_, _>>()
    }

    fn de_insert_trait(&self, r#trait: TypedTraitDeclaration) -> DeclarationId {
        let span = r#trait.name.span();
        DeclarationId::new(self.slab.insert(DeclarationWrapper::Trait(r#trait)), span)
    }

    fn de_get_trait(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TypedTraitDeclaration, CompileError> {
        self.slab.get(*index).expect_trait(span)
    }

    fn de_insert_trait_fn(&self, trait_fn: TypedTraitFn) -> DeclarationId {
        let span = trait_fn.name.span();
        DeclarationId::new(
            self.slab.insert(DeclarationWrapper::TraitFn(trait_fn)),
            span,
        )
    }

    fn de_get_trait_fn(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TypedTraitFn, CompileError> {
        self.slab.get(*index).expect_trait_fn(span)
    }

    fn insert_trait_impl(&self, trait_impl: TypedImplTrait) -> DeclarationId {
        let span = trait_impl.span.clone();
        DeclarationId::new(
            self.slab.insert(DeclarationWrapper::TraitImpl(trait_impl)),
            span,
        )
    }

    fn de_get_trait_impl(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TypedImplTrait, CompileError> {
        self.slab.get(*index).expect_trait_impl(span)
    }

    fn de_insert_struct(&self, r#struct: TypedStructDeclaration) -> DeclarationId {
        let span = r#struct.span();
        DeclarationId::new(self.slab.insert(DeclarationWrapper::Struct(r#struct)), span)
    }

    fn de_get_struct(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TypedStructDeclaration, CompileError> {
        self.slab.get(*index).expect_struct(span)
    }

    fn de_add_monomorphized_struct_copy(
        &self,
        original_id: DeclarationId,
        new_copy: TypedStructDeclaration,
    ) {
        let span = new_copy.span();
        let new_id =
            DeclarationId::new(self.slab.insert(DeclarationWrapper::Struct(new_copy)), span);
        self.de_add_monomorphized_copy(original_id, new_id)
    }

    fn de_get_monomorphized_struct_copies(
        &self,
        original_id: DeclarationId,
        span: &Span,
    ) -> Result<Vec<TypedStructDeclaration>, CompileError> {
        self.de_get_monomorphized_copies(original_id)
            .into_iter()
            .map(|x| x.expect_struct(span))
            .collect::<Result<_, _>>()
    }
}

pub(crate) fn de_clear() {
    DECLARATION_ENGINE.clear()
}

pub(crate) fn de_look_up_decl_id(index: DeclarationId) -> DeclarationWrapper {
    DECLARATION_ENGINE.de_look_up_decl_id(index)
}

pub(crate) fn de_insert_function(function: TypedFunctionDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.de_insert_function(function)
}

pub(crate) fn de_get_function(
    index: DeclarationId,
    span: &Span,
) -> Result<TypedFunctionDeclaration, CompileError> {
    DECLARATION_ENGINE.de_get_function(index, span)
}

pub(crate) fn de_add_monomorphized_function_copy(
    original_id: DeclarationId,
    new_copy: TypedFunctionDeclaration,
) {
    DECLARATION_ENGINE.de_add_monomorphized_function_copy(original_id, new_copy);
}

pub(crate) fn de_get_monomorphized_function_copies(
    original_id: DeclarationId,
    span: &Span,
) -> Result<Vec<TypedFunctionDeclaration>, CompileError> {
    DECLARATION_ENGINE.de_get_monomorphized_function_copies(original_id, span)
}

pub(crate) fn de_insert_trait(r#trait: TypedTraitDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.de_insert_trait(r#trait)
}

pub(crate) fn de_get_trait(
    index: DeclarationId,
    span: &Span,
) -> Result<TypedTraitDeclaration, CompileError> {
    DECLARATION_ENGINE.de_get_trait(index, span)
}

pub(crate) fn de_insert_trait_fn(trait_fn: TypedTraitFn) -> DeclarationId {
    DECLARATION_ENGINE.de_insert_trait_fn(trait_fn)
}

pub(crate) fn de_get_trait_fn(
    index: DeclarationId,
    span: &Span,
) -> Result<TypedTraitFn, CompileError> {
    DECLARATION_ENGINE.de_get_trait_fn(index, span)
}

pub(crate) fn insert_trait_impl(trait_impl: TypedImplTrait) -> DeclarationId {
    DECLARATION_ENGINE.insert_trait_impl(trait_impl)
}

pub(crate) fn de_get_trait_impl(
    index: DeclarationId,
    span: &Span,
) -> Result<TypedImplTrait, CompileError> {
    DECLARATION_ENGINE.de_get_trait_impl(index, span)
}

pub(crate) fn de_insert_struct(r#struct: TypedStructDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.de_insert_struct(r#struct)
}

pub(crate) fn de_get_struct(
    index: DeclarationId,
    span: &Span,
) -> Result<TypedStructDeclaration, CompileError> {
    DECLARATION_ENGINE.de_get_struct(index, span)
}

pub(crate) fn de_add_monomorphized_struct_copy(
    original_id: DeclarationId,
    new_copy: TypedStructDeclaration,
) {
    DECLARATION_ENGINE.de_add_monomorphized_struct_copy(original_id, new_copy);
}

pub(crate) fn de_get_monomorphized_struct_copies(
    original_id: DeclarationId,
    span: &Span,
) -> Result<Vec<TypedStructDeclaration>, CompileError> {
    DECLARATION_ENGINE.de_get_monomorphized_struct_copies(original_id, span)
}
