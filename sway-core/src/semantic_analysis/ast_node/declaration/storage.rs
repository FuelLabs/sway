use crate::{
    error::*,
    fuel_prelude::fuel_tx::StorageSlot,
    ir_generation::{
        const_eval::compile_constant_expression_to_constant, storage::serialize_to_storage_slots,
    },
    language::ty,
    metadata::MetadataManager,
    Engines,
};
use sway_error::error::CompileError;
use sway_ir::{Context, Module};
use sway_types::state::StateIndex;

impl ty::TyStorageDecl {
    pub(crate) fn get_initialized_storage_slots(
        &self,
        engines: Engines<'_>,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
        experimental_storage: bool,
    ) -> CompileResult<Vec<StorageSlot>> {
        let mut errors = vec![];
        let storage_slots = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                f.get_initialized_storage_slots(
                    engines,
                    context,
                    md_mgr,
                    module,
                    &StateIndex::new(i),
                    experimental_storage,
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
        engines: Engines<'_>,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
        ix: &StateIndex,
        experimental_storage: bool,
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
            serialize_to_storage_slots(
                &constant,
                context,
                ix,
                &constant.ty,
                &[],
                experimental_storage,
            )
        })
    }
}
