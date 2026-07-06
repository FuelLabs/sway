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

            let mut type_sizes_and_slots = self
                .fields
                .iter()
                .map(|field| {
                    let type_size_and_slots =
                        field.get_initialized_storage_slots(engines, context, md_mgr, module);

                    // Check if the slot with the same key was already used.
                    if let Ok((_type_size_in_bytes, slots)) = &type_size_and_slots {
                        for slot in slots.iter() {
                            if let Some(old_field) = slot_fields.insert(*slot.key(), field.clone())
                            {
                                handler.emit_warn(CompileWarning {
                                    span: field.span(),
                                    warning_content:
                                        sway_error::warning::Warning::DuplicatedStorageKey {
                                            first_field: (&old_field.name).into(),
                                            first_field_full_name: old_field.full_name(),
                                            first_field_key_is_compiler_generated: old_field
                                                .key_expression
                                                .is_none(),
                                            second_field: (&field.name).into(),
                                            second_field_full_name: field.full_name(),
                                            second_field_key_is_compiler_generated: field
                                                .key_expression
                                                .is_none(),
                                            key: format!("0x{:x}", slot.key()),
                                        },
                                })
                            }
                        }
                    }

                    type_size_and_slots
                })
                .filter_map(|res| res.map_err(|e| handler.emit_err(e)).ok())
                .collect::<Vec<_>>();

            // TODO: (INIT-STORAGE) Implement initialization of storage fields
            //       for dynamic storage. For now, we just ignore storage fields
            //       whose type size is larger than 32 bytes, and don't initialize them.
            if context.experimental.dynamic_storage {
                type_sizes_and_slots
                    .retain(|(type_size_in_bytes, _slots)| *type_size_in_bytes <= 32);
            }

            let storage_slots = type_sizes_and_slots
                .into_iter()
                .flat_map(|(_type_size_in_bytes, slots)| slots)
                .collect();

            Ok(storage_slots)
        })
    }
}

impl ty::TyStorageField {
    /// Returns a tuple containing the `self` storage field's type size in bytes,
    /// and the storage slots that contain the value defined in `self.initializer`.
    pub(crate) fn get_initialized_storage_slots(
        &self,
        engines: &Engines,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> Result<(u64, Vec<StorageSlot>), CompileError> {
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
        .map(|constant| serialize_to_storage_slots(context, &constant, &self.path(), key))
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
            if let ConstantValue::B256(key) = const_key.get_content(context).value.clone() {
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
