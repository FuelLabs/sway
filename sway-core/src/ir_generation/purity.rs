use crate::{
    language::{
        promote_purity,
        Purity::{self, *},
    },
    metadata::MetadataManager,
};

use sway_error::{error::CompileError, handler::Handler};
use sway_error::{
    error::StorageAccess,
    warning::{CompileWarning, Warning},
};
use sway_ir::{Context, FuelVmInstruction, Function, InstOp};
use sway_types::span::Span;

use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PurityEnv {
    memos: HashMap<Function, (bool, bool)>,
}

/// Analyses purity annotations on functions.
///
/// Designed to be called for each entry point, _prior_ to inlining or other optimizations.
/// The checker will check this function and any that it calls.
///
/// Returns bools for whether it (reads, writes).
pub(crate) fn check_function_purity(
    handler: &Handler,
    env: &mut PurityEnv,
    context: &Context,
    md_mgr: &mut MetadataManager,
    function: &Function,
) -> (bool, bool) {
    // Iterate for each instruction in the function and gather whether we have read and/or
    // write storage operations:
    // - via the storage IR instructions,
    // - via ASM blocks with storage VM instructions, or
    // - via calls into functions with the above.
    let attributed_purity = md_mgr.md_to_purity(context, function.get_metadata(context));

    let mut storage_access_violations = vec![];
    let (reads, writes) = function.instruction_iter(context).fold(
        (false, false),
        |(reads, writes), (_block, ins_value)| {
            ins_value
                .get_instruction(context)
                .map(|instruction| {
                    match &instruction.op {
                        InstOp::FuelVm(inst) if is_store_access_fuel_vm_instruction(inst) => {
                            let storage_access = store_access_fuel_vm_instruction_to_storage_access(inst);
                            if violates_purity(&storage_access, &attributed_purity) {
                                // When compiling Sway code, the only way to get FuelVM store access instructions in the IR
                                // is via store access intrinsics. So we know that the span stored in the metadata will be
                                // the intrinsic's call span which is suitable for error reporting.
                                let intrinsic_call_span = md_mgr.md_to_span(context, ins_value.get_metadata(context)).unwrap_or(Span::dummy());
                                storage_access_violations.push((intrinsic_call_span, storage_access));
                            }

                            match inst {
                                FuelVmInstruction::StateLoadQuadWord { .. }
                                | FuelVmInstruction::StateLoadWord { .. } => (true, writes),
                                FuelVmInstruction::StateClear { .. }
                                | FuelVmInstruction::StateStoreQuadWord { .. }
                                | FuelVmInstruction::StateStoreWord { .. } => (reads, true),
                                _ => unreachable!("The FuelVM instruction is checked to be a store access instruction."),
                            }
                        }

                        // Iterate for and check each instruction in the ASM block.
                        InstOp::AsmBlock(asm_block, _args) => asm_block.body.iter().fold(
                            (reads, writes),
                            |(reads, writes), asm_op| {
                                let inst = asm_op.op_name.as_str();
                                if is_store_access_asm_instruction(inst) {
                                    let storage_access = store_access_asm_instruction_to_storage_access(inst);
                                    if violates_purity(&storage_access, &attributed_purity) {
                                        let asm_inst_span = md_mgr.md_to_span(context, asm_op.metadata).unwrap_or(Span::dummy());
                                        storage_access_violations.push((asm_inst_span, storage_access));
                                    }

                                    match inst {
                                        "srw" | "srwq" => (true, writes),
                                        "scwq" | "sww" | "swwq" => (reads, true),
                                        _ => unreachable!("The ASM instruction is checked to be a store access instruction."),
                                    }
                                } else {
                                    (reads, writes)
                                }
                            }
                        ),

                        // Recurse to find the called function purity.  Use memoisation to
                        // avoid redoing work.
                        InstOp::Call(callee, _args) => {
                            let (callee_reads, callee_writes) =
                                env.memos.get(callee).copied().unwrap_or_else(|| {
                                    let r_w = check_function_purity(
                                        handler, env, context, md_mgr, callee,
                                    );
                                    env.memos.insert(*callee, r_w);
                                    r_w
                                });
                            if callee_reads || callee_writes {
                                let callee_span = md_mgr.md_to_fn_call_path_span(context, ins_value.get_metadata(context)).unwrap_or(Span::dummy());
                                let storage_access = StorageAccess::ImpureFunctionCall(callee_span.clone(), callee_reads, callee_writes);
                                if violates_purity(&storage_access, &attributed_purity) {
                                    storage_access_violations.push((callee_span, storage_access));
                                }
                            }
                            (reads || callee_reads, writes || callee_writes)
                        }

                        _otherwise => (reads, writes),
                    }
                })
                .unwrap_or_else(|| (reads, writes))
        },
    );

    // Simple closures for each of the error types.
    let error = |span: Span, needed| {
        // We don't emit errors on the generated `__entry` function
        // but do on the original entry functions and all other functions.
        if !function.is_entry(context) || function.is_original_entry(context) {
            handler.emit_err(CompileError::StorageAccessMismatched {
                span,
                is_pure: matches!(attributed_purity, Pure),
                suggested_attributes: promote_purity(attributed_purity, needed)
                    .to_attribute_syntax(),
                storage_access_violations,
            });
        }
    };

    let warn = |span, purity: Purity| {
        // Do not warn on generated code
        if span != Span::dummy() {
            handler.emit_warn(CompileWarning {
                warning_content: Warning::DeadStorageDeclarationForFunction {
                    unneeded_attrib: purity.to_attribute_syntax(),
                },
                span,
            });
        }
    };

    let span = md_mgr
        .md_to_fn_name_span(context, function.get_metadata(context))
        .unwrap_or_else(Span::dummy);

    match (attributed_purity, reads, writes) {
        // Has no attributes but needs some.
        (Pure, true, false) => error(span, Reads),
        (Pure, false, true) => error(span, Writes),
        (Pure, true, true) => error(span, ReadsWrites),

        // Or the attribute must match the behavior.
        (Reads, _, true) => error(span, Writes),

        // Or we have unneeded attributes.
        (ReadsWrites, false, true) => warn(span, Reads),
        (ReadsWrites, true, false) => warn(span, Writes),
        (ReadsWrites, false, false) => warn(span, ReadsWrites),
        (Reads, false, false) => warn(span, Reads),
        (Writes, _, false) => warn(span, Writes),

        // Attributes and effects are in total agreement.
        (Pure, false, false)
        | (Reads, true, false)
        | (Writes, _, true) // storage(write) allows reading as well 
        | (ReadsWrites, true, true) => (),
    };

    (reads, writes)
}

fn is_store_access_fuel_vm_instruction(inst: &FuelVmInstruction) -> bool {
    matches!(
        inst,
        FuelVmInstruction::StateLoadWord { .. }
            | FuelVmInstruction::StateLoadQuadWord { .. }
            | FuelVmInstruction::StateClear { .. }
            | FuelVmInstruction::StateStoreWord { .. }
            | FuelVmInstruction::StateStoreQuadWord { .. }
    )
}

fn store_access_fuel_vm_instruction_to_storage_access(inst: &FuelVmInstruction) -> StorageAccess {
    match inst {
        FuelVmInstruction::StateLoadWord { .. } => StorageAccess::ReadWord,
        FuelVmInstruction::StateLoadQuadWord { .. } => StorageAccess::ReadSlots,
        FuelVmInstruction::StateClear { .. } => StorageAccess::Clear,
        FuelVmInstruction::StateStoreWord { .. } => StorageAccess::WriteWord,
        FuelVmInstruction::StateStoreQuadWord { .. } => StorageAccess::WriteSlots,
        _ => panic!("The FuelVM instruction is not a store access instruction."),
    }
}

fn is_store_access_asm_instruction(inst: &str) -> bool {
    matches!(inst, "srw" | "srwq" | "scwq" | "sww" | "swwq")
}

fn store_access_asm_instruction_to_storage_access(inst: &str) -> StorageAccess {
    match inst {
        "srw" => StorageAccess::ReadWord,
        "srwq" => StorageAccess::ReadSlots,
        "scwq" => StorageAccess::Clear,
        "sww" => StorageAccess::WriteWord,
        "swwq" => StorageAccess::WriteSlots,
        _ => panic!("The ASM instruction \"{inst}\" is not a store access instruction."),
    }
}

/// Returns true if the `storage_access` violates the given expected `purity`.
fn violates_purity(storage_access: &StorageAccess, purity: &Purity) -> bool {
    match purity {
        Pure => true,
        Reads => storage_access.is_write(),
        Writes => false,
        ReadsWrites => false,
    }
}
