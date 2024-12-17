use super::{MigrationStep, MigrationStepKind};
use crate::migrations::ProgramInfo;
use anyhow::{Ok, Result};
use sway_core::language::ty::TyDecl;
use sway_types::Span;

pub(super) const REVIEW_STORAGE_SLOT_KEYS_STEP: MigrationStep = MigrationStep {
    title: "Review explicitly defined slot keys in storage declarations (`in` keywords)",
    duration: 2,
    kind: MigrationStepKind::Instruction(review_storage_slot_keys_step),
    help: &[
        "If the slot keys used in `in` keywords represent keys generated for `storage` fields",
        "by the Sway compiler, those keys might need to be recalculated.",
        " ",
        "The previous formula for calculating storage field keys was: `sha256(\"storage.<field name>\")`.",
        "The new formula is:                                          `sha256((0u8, \"storage.<field name>\"))`.",
    ],
};

fn review_storage_slot_keys_step(program_info: &ProgramInfo) -> Result<Vec<Span>> {
    let mut res = vec![];

    let program = &program_info.ty_program;
    let engines = program_info.engines;

    // Storage can be declared only in the entry point of a contract and there can be
    // only one storage declaration per program.
    if let Some(TyDecl::StorageDecl(storage_decl)) = program
        .declarations
        .iter()
        .find(|decl| matches!(decl, TyDecl::StorageDecl(_)))
    {
        let storage_decl = engines.de().get_storage(&storage_decl.decl_id);

        for key_expression in storage_decl
            .fields
            .iter()
            .filter_map(|storage_field| storage_field.key_expression.as_ref())
        {
            res.push(key_expression.span.clone());
        }
    }

    Ok(res)
}
