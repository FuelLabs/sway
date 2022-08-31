use lazy_static::lazy_static;
use std::{collections::HashMap, sync::RwLock};

use crate::{
    concurrent_slab::ConcurrentSlab,
    semantic_analysis::{
        TypedImplTrait, TypedStructDeclaration, TypedTraitDeclaration, TypedTraitFn,
    },
    TypedFunctionDeclaration,
};

use super::{declaration_id::DeclarationId, declaration_wrapper::DeclarationWrapper};

lazy_static! {
    static ref DECLARATION_ENGINE: DeclarationEngine = DeclarationEngine::default();
}

/// Used inside of type inference to store declarations.
#[derive(Debug, Default)]
pub struct DeclarationEngine {
    slab: ConcurrentSlab<DeclarationId, DeclarationWrapper>,
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
        self.slab.get(index)
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
            Some(copies) => copies.into_iter().map(|copy| self.slab.get(copy)).collect(),
            None => vec![],
        }
    }

    fn de_insert_function(&self, function: TypedFunctionDeclaration) -> DeclarationId {
        self.slab.insert(DeclarationWrapper::Function(function))
    }

    fn de_get_function(
        &self,
        index: DeclarationId,
    ) -> Result<TypedFunctionDeclaration, DeclarationWrapper> {
        self.slab.get(index).expect_function()
    }

    fn de_add_monomorphized_function_copy(
        &self,
        original_id: DeclarationId,
        new_copy: TypedFunctionDeclaration,
    ) {
        let new_id = self.slab.insert(DeclarationWrapper::Function(new_copy));
        self.de_add_monomorphized_copy(original_id, new_id)
    }

    fn de_get_monomorphized_function_copies(
        &self,
        original_id: DeclarationId,
    ) -> Result<Vec<TypedFunctionDeclaration>, DeclarationWrapper> {
        self.de_get_monomorphized_copies(original_id)
            .into_iter()
            .map(|x| x.expect_function())
            .collect::<Result<_, _>>()
    }

    fn de_insert_trait(&self, r#trait: TypedTraitDeclaration) -> DeclarationId {
        self.slab.insert(DeclarationWrapper::Trait(r#trait))
    }

    fn de_get_trait(
        &self,
        index: DeclarationId,
    ) -> Result<TypedTraitDeclaration, DeclarationWrapper> {
        self.slab.get(index).expect_trait()
    }

    fn de_insert_trait_fn(&self, trait_fn: TypedTraitFn) -> DeclarationId {
        self.slab.insert(DeclarationWrapper::TraitFn(trait_fn))
    }

    fn de_get_trait_fn(&self, index: DeclarationId) -> Result<TypedTraitFn, DeclarationWrapper> {
        self.slab.get(index).expect_trait_fn()
    }

    fn insert_trait_impl(&self, trait_impl: TypedImplTrait) -> DeclarationId {
        self.slab.insert(DeclarationWrapper::TraitImpl(trait_impl))
    }

    fn de_get_trait_impl(
        &self,
        index: DeclarationId,
    ) -> Result<TypedImplTrait, DeclarationWrapper> {
        self.slab.get(index).expect_trait_impl()
    }

    fn de_insert_struct(&self, r#struct: TypedStructDeclaration) -> DeclarationId {
        self.slab.insert(DeclarationWrapper::Struct(r#struct))
    }

    fn de_get_struct(
        &self,
        index: DeclarationId,
    ) -> Result<TypedStructDeclaration, DeclarationWrapper> {
        self.slab.get(index).expect_struct()
    }

    fn de_add_monomorphized_struct_copy(
        &self,
        original_id: DeclarationId,
        new_copy: TypedStructDeclaration,
    ) {
        let new_id = self.slab.insert(DeclarationWrapper::Struct(new_copy));
        self.de_add_monomorphized_copy(original_id, new_id)
    }

    fn de_get_monomorphized_struct_copies(
        &self,
        original_id: DeclarationId,
    ) -> Result<Vec<TypedStructDeclaration>, DeclarationWrapper> {
        self.de_get_monomorphized_copies(original_id)
            .into_iter()
            .map(|x| x.expect_struct())
            .collect::<Result<_, _>>()
    }
}

pub(crate) fn de_clear() {
    DECLARATION_ENGINE.clear()
}

pub(crate) fn de_look_up_decl_id(index: DeclarationId) -> DeclarationWrapper {
    DECLARATION_ENGINE.de_look_up_decl_id(index)
}
