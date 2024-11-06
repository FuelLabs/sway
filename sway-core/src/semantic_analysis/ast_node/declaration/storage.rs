use std::collections::HashMap;

use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    fuel_prelude::fuel_tx::StorageSlot,
    ir_generation::{
        const_eval::compile_constant_expression_to_constant, storage::serialize_to_storage_slots,
    },
    language::{
        parsed::StorageDeclaration,
        ty::{self, TyExpression, TyStorageField},
    },
    metadata::MetadataManager,
    semantic_analysis::{
        symbol_collection_context::SymbolCollectionContext, TypeCheckAnalysis,
        TypeCheckAnalysisContext, TypeCheckFinalization, TypeCheckFinalizationContext,
    },
    Engines,
};
use fuel_vm::fuel_tx::Bytes32;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::CompileWarning,
};
use sway_ir::{ConstantValue, Context, Module};
use sway_types::{u256::U256, Spanned};

impl ty::TyStorageDecl {
    pub(crate) fn collect(
        _handler: &Handler,
        _engines: &Engines,
        _ctx: &mut SymbolCollectionContext,
        _decl_id: &ParsedDeclId<StorageDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }

    pub(crate) fn get_initialized_storage_slots(
        &self,
        handler: &Handler,
        engines: &Engines,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> Result<Vec<StorageSlot>, ErrorEmitted> {
        handler.scope(|handler| {
            let mut slot_fields = HashMap::<Bytes32, TyStorageField>::new();
            let storage_slots = self
                .fields
                .iter()
                .map(|f| {
                    let slots = f.get_initialized_storage_slots(engines, context, md_mgr, module);

                    // Check if slot with same key was already used and throw warning.
                    if let Ok(slots) = &slots {
                        for s in slots.iter() {
                            if let Some(old_field) = slot_fields.insert(*s.key(), f.clone()) {
                                handler.emit_warn(CompileWarning {
                                    span: f.span(),
                                    warning_content:
                                        sway_error::warning::Warning::DuplicatedStorageKey {
                                            first_field: (&old_field.name).into(),
                                            first_field_full_name: old_field.full_name(),
                                            first_field_key_is_compiler_generated: old_field
                                                .key_expression
                                                .is_none(),
                                            second_field: (&f.name).into(),
                                            second_field_full_name: f.full_name(),
                                            second_field_key_is_compiler_generated: f
                                                .key_expression
                                                .is_none(),
                                            key: format!("0x{:x}", s.key()),
                                            experimental_storage_domains: context
                                                .experimental
                                                .storage_domains,
                                        },
                                })
                            }
                        }
                    }
                    slots
                })
                .filter_map(|s| s.map_err(|e| handler.emit_err(e)).ok())
                .flatten()
                .collect::<Vec<_>>();

            Ok(storage_slots)
        })
    }
}

impl ty::TyStorageField {
    pub(crate) fn get_initialized_storage_slots(
        &self,
        engines: &Engines,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> Result<Vec<StorageSlot>, CompileError> {
        let key =
            Self::get_key_expression_const(&self.key_expression, engines, context, md_mgr, module)?;
        compile_constant_expression_to_constant(
            engines,
            context,
            md_mgr,
            module,
            None,
            None,
            &self.initializer,
        )
        .map(|constant| {
            serialize_to_storage_slots(
                &constant,
                context,
                self.namespace_names
                    .iter()
                    .map(|i| i.as_str().to_string())
                    .chain(vec![self.name.as_str().to_string()])
                    .collect(),
                key,
                &constant.ty,
            )
        })
    }

    pub(crate) fn get_key_expression_const(
        key_expression: &Option<TyExpression>,
        engines: &Engines,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> Result<Option<U256>, CompileError> {
        if let Some(key_expression) = key_expression {
            let const_key = compile_constant_expression_to_constant(
                engines,
                context,
                md_mgr,
                module,
                None,
                None,
                key_expression,
            )?;
            if let ConstantValue::B256(key) = const_key.value {
                Ok(Some(key))
            } else {
                Err(CompileError::Internal(
                    "Storage keys must have type \"b256\".",
                    key_expression.span.clone(),
                ))
            }
        } else {
            Ok(None)
        }
    }
}

impl TypeCheckAnalysis for ty::TyStorageDecl {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            for field in self.fields.iter() {
                let _ = field.type_check_analyze(handler, ctx);
            }
            Ok(())
        })
    }
}

impl TypeCheckAnalysis for ty::TyStorageField {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        self.initializer.type_check_analyze(handler, ctx)
    }
}

impl TypeCheckFinalization for ty::TyStorageDecl {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            for field in self.fields.iter_mut() {
                let _ = field.type_check_finalize(handler, ctx);
            }
            Ok(())
        })
    }
}

impl TypeCheckFinalization for ty::TyStorageField {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        self.initializer.type_check_finalize(handler, ctx)
    }
}
