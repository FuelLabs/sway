use crate::{
    fuel_prelude::fuel_tx::StorageSlot,
    ir_generation::{
        const_eval::compile_constant_expression_to_constant, storage::serialize_to_storage_slots,
    },
    language::ty,
    metadata::MetadataManager,
    semantic_analysis::{
        TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckFinalization,
        TypeCheckFinalizationContext,
    },
    Engines,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_ir::{Context, Module};
use sway_types::u256::U256;

impl ty::TyStorageDecl {
    pub(crate) fn get_initialized_storage_slots(
        &self,
        handler: &Handler,
        engines: &Engines,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> Result<Vec<StorageSlot>, ErrorEmitted> {
        handler.scope(|handler| {
            let storage_slots = self
                .fields
                .iter()
                .map(|f| {
                    f.get_initialized_storage_slots(
                        engines,
                        context,
                        md_mgr,
                        module,
                        vec![f.name.as_str().to_string()],
                        f.key.clone(),
                    )
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
        storage_field_names: Vec<String>,
        key: Option<U256>,
    ) -> Result<Vec<StorageSlot>, CompileError> {
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
            serialize_to_storage_slots(&constant, context, storage_field_names, key, &constant.ty)
        })
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
