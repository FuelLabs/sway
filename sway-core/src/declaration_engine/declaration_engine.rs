use lazy_static::lazy_static;
use std::{collections::HashMap, sync::RwLock};
use sway_types::{Span, Spanned};

use crate::{
    concurrent_slab::ConcurrentSlab,
    error::{err, ok},
    namespace::{Path, Root},
    semantic_analysis::{
        TypedImplTrait, TypedStorageDeclaration, TypedStructDeclaration, TypedTraitDeclaration,
        TypedTraitFn,
    },
    type_system::TypeArgument,
    type_system::{type_engine::monomorphize, EnforceTypeArguments},
    CompileError, CompileResult, MonomorphizeHelper, TypedFunctionDeclaration,
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
    is_monomorph_cache_enabled: bool,
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
        monomorphized_copies
            .entry(*original_id)
            .and_modify(|f| f.push(new_id.clone()))
            .or_insert_with(|| vec![new_id.clone()]);
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

    fn de_insert_trait_impl(&self, trait_impl: TypedImplTrait) -> DeclarationId {
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

    pub(crate) fn de_get_monomorphized_decl(
        &self,
        original_id: DeclarationId,
        type_arguments: &Vec<TypeArgument>,
        span: &Span,
    ) -> Result<DeclarationWrapper, CompileError> {
        for monomorphized_decl in self
            .de_get_monomorphized_copies(original_id)
            .iter()
            .cloned()
        {
            if monomorphized_decl.type_parameters() == type_arguments {
                return Ok(monomorphized_decl);
            }
        }
        Err(CompileError::Internal(
            "could not find monomorphized decl",
            span.clone(),
        ))
    }

    pub(crate) fn de_get_or_create_monomorphized_decl<T>(
        &self,
        decl_id: DeclarationId,
        type_arguments: &mut Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        call_site_span: &Span,
        namespace: &Root,
        module_path: &Path,
    ) -> CompileResult<DeclarationWrapper>
    where
        T: MonomorphizeHelper,
    {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        if self.is_monomorph_cache_enabled {
            let cached_decl = check!(
                CompileResult::from(self.de_get_monomorphized_decl(
                    decl_id,
                    type_arguments,
                    call_site_span
                )),
                return err(warnings, errors),
                warnings,
                errors
            );
            return ok(cached_decl, warnings, errors);
        }

        // monomorphize the declaration into a new copy
        let mut typed_declaration = self.slab.get(*decl_id);

        check!(
            monomorphize(
                &mut typed_declaration,
                type_arguments,
                enforce_type_arguments,
                call_site_span,
                namespace,
                module_path,
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // add the new copy to the declaration engine
        let new_id = self.slab.insert(typed_declaration.to_wrapper());
        self.de_add_monomorphized_copy(decl_id, DeclarationId::new(new_id, call_site_span.clone()));

        ok(typed_declaration, warnings, errors)
    }

    fn de_insert_storage(&self, storage: TypedStorageDeclaration) -> DeclarationId {
        let span = storage.span();
        DeclarationId::new(self.slab.insert(DeclarationWrapper::Storage(storage)), span)
    }

    fn de_get_storage(
        &self,
        index: DeclarationId,
        span: &Span,
    ) -> Result<TypedStorageDeclaration, CompileError> {
        self.slab.get(*index).expect_storage(span)
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

pub(crate) fn de_insert_trait_impl(trait_impl: TypedImplTrait) -> DeclarationId {
    DECLARATION_ENGINE.de_insert_trait_impl(trait_impl)
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

pub fn de_get_struct(
    index: DeclarationId,
    span: &Span,
) -> Result<TypedStructDeclaration, CompileError> {
    DECLARATION_ENGINE.de_get_struct(index, span)
}

pub(crate) fn de_insert_storage(storage: TypedStorageDeclaration) -> DeclarationId {
    DECLARATION_ENGINE.de_insert_storage(storage)
}

pub fn de_get_storage(
    index: DeclarationId,
    span: &Span,
) -> Result<TypedStorageDeclaration, CompileError> {
    DECLARATION_ENGINE.de_get_storage(index, span)
}

pub(crate) fn de_get_or_create_monomorphized_decl<T>(
    decl_id: DeclarationId,
    type_arguments: &mut Vec<TypeArgument>,
    enforce_type_arguments: EnforceTypeArguments,
    call_site_span: &Span,
    namespace: &Root,
    module_path: &Path,
) -> CompileResult<DeclarationWrapper>
where
    T: MonomorphizeHelper,
{
    DECLARATION_ENGINE.de_get_or_create_monomorphized_decl::<T>(
        decl_id,
        type_arguments,
        enforce_type_arguments,
        call_site_span,
        namespace,
        module_path,
    )
}
