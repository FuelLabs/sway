use std::collections::HashMap;

use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    fuel_prelude::fuel_tx::StorageSlot,
    ir_generation::{
        const_eval::compile_constant_expression_to_constant,
        storage::{get_storage_key_string, serialize_to_storage_slots},
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
                    if let Ok(slots) = slots.clone() {
                        for s in slots.into_iter() {
                            if let Some(old_field) = slot_fields.insert(*s.key(), f.clone()) {
                                handler.emit_warn(CompileWarning {
                                    span: f.span(),
                                    warning_content:
                                        sway_error::warning::Warning::DuplicatedStorageKey {
                                            key: format!("{:X} ", s.key()),
                                            field1: get_storage_key_string(
                                                old_field
                                                    .namespace_names
                                                    .iter()
                                                    .map(|i| i.as_str().to_string())
                                                    .chain(vec![old_field
                                                        .name
                                                        .as_str()
                                                        .to_string()])
                                                    .collect::<Vec<_>>(),
                                            ),
                                            field2: get_storage_key_string(
                                                f.namespace_names
                                                    .iter()
                                                    .map(|i| i.as_str().to_string())
                                                    .chain(vec![f.name.as_str().to_string()])
                                                    .collect::<Vec<_>>(),
                                            ),
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
                    "Expected B256 key",
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
