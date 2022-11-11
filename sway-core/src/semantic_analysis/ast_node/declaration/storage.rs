use crate::{
    error::*,
    fuel_prelude::fuel_tx::StorageSlot,
    ir_generation::{
        const_eval::compile_constant_expression_to_constant, storage::serialize_to_storage_slots,
    },
    language::ty,
    metadata::MetadataManager,
    TypeEngine,
};
use sway_error::error::CompileError;
use sway_ir::{Context, Module};
use sway_types::state::StateIndex;

impl ty::TyStorageDeclaration {
    pub(crate) fn get_initialized_storage_slots(
        &self,
        type_engine: &TypeEngine,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> CompileResult<Vec<StorageSlot>> {
        let mut errors = vec![];
        let storage_slots = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                f.get_initialized_storage_slots(
                    type_engine,
                    context,
                    md_mgr,
                    module,
                    &StateIndex::new(i),
                )
            })
            .filter_map(|s| s.map_err(|e| errors.push(e)).ok())
            .flatten()
            .collect::<Vec<_>>();

        match errors.is_empty() {
            true => ok(storage_slots, vec![], vec![]),
            false => err(vec![], errors),
        }
    }
}

impl ty::TyStorageField {
    pub(crate) fn get_initialized_storage_slots(
        &self,
        type_engine: &TypeEngine,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
        ix: &StateIndex,
    ) -> Result<Vec<StorageSlot>, CompileError> {
        compile_constant_expression_to_constant(
            type_engine,
            context,
            md_mgr,
            module,
            None,
            &self.initializer,
        )
        .map(|constant| serialize_to_storage_slots(&constant, context, ix, &constant.ty, &[]))
    }
}
