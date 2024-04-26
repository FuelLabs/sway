use crate::{
    language::{
        promote_purity,
        Purity::{self, *},
    },
    metadata::{MetadataManager, StorageOperation},
};

use sway_error::warning::{CompileWarning, Warning};
use sway_error::{error::CompileError, handler::Handler};
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
    // - via ASM blocks with storage VM instructions or
    // - via calls into functions with the above.
    let (reads, writes) = function.instruction_iter(context).fold(
        (false, false),
        |(reads, writes), (_block, ins_value)| {
            ins_value
                .get_instruction(context)
                .map(|instruction| {
                    match &instruction.op {
                        InstOp::FuelVm(FuelVmInstruction::StateLoadQuadWord { .. })
                        | InstOp::FuelVm(FuelVmInstruction::StateLoadWord(_)) => (true, writes),

                        InstOp::FuelVm(FuelVmInstruction::StateClear { .. })
                        | InstOp::FuelVm(FuelVmInstruction::StateStoreQuadWord { .. })
                        | InstOp::FuelVm(FuelVmInstruction::StateStoreWord { .. }) => (reads, true),

                        // Iterate for and check each instruction in the ASM block.
                        InstOp::AsmBlock(asm_block, _args) => asm_block.body.iter().fold(
                            (reads, writes),
                            |(reads, writes), asm_op| match asm_op.op_name.as_str() {
                                "scwq" | "srw" | "srwq" => (true, writes),
                                "sww" | "swwq" => (reads, true),
                                _ => (reads, writes),
                            },
                        ),

                        // Recurse to find the called function purity.  Use memoisation to
                        // avoid redoing work.
                        InstOp::Call(callee, _args) => {
                            let (called_fn_reads, called_fn_writes) =
                                env.memos.get(callee).copied().unwrap_or_else(|| {
                                    let r_w = check_function_purity(
                                        handler, env, context, md_mgr, callee,
                                    );
                                    env.memos.insert(*callee, r_w);
                                    r_w
                                });
                            (reads || called_fn_reads, writes || called_fn_writes)
                        }

                        _otherwise => (reads, writes),
                    }
                })
                .unwrap_or_else(|| (reads, writes))
        },
    );

    let attributed_purity = md_mgr.md_to_storage_op(context, function.get_metadata(context));
    let span = md_mgr
        .md_to_span(context, function.get_metadata(context))
        .unwrap_or_else(Span::dummy);

    // Simple closures for each of the error types.
    let error = |span, storage_op, existing, needed| {
        handler.emit_err(CompileError::ImpureInPureContext {
            storage_op,
            attrs: promote_purity(existing, needed).to_attribute_syntax(),
            span,
        });
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

    match (attributed_purity, reads, writes) {
        // Has no attributes but needs some.
        (None, true, false) => error(span, "read", Pure, Reads),
        (None, false, true) => error(span, "write", Pure, Writes),
        (None, true, true) => error(span, "read & write", Pure, ReadsWrites),

        // Or the attribute must match the behaviour.
        (Some(StorageOperation::Reads), _, true) => error(span, "write", Reads, Writes),

        // Or we have unneeded attributes.
        (Some(StorageOperation::ReadsWrites), false, true) => warn(span, Reads),
        (Some(StorageOperation::ReadsWrites), true, false) => warn(span, Writes),
        (Some(StorageOperation::ReadsWrites), false, false) => warn(span, ReadsWrites),
        (Some(StorageOperation::Reads), false, false) => warn(span, Reads),
        (Some(StorageOperation::Writes), _, false) => warn(span, Writes),

        // Attributes and effects are in total agreement
        (None, false, false)
        | (Some(StorageOperation::Reads), true, false)
        | (Some(StorageOperation::Writes), _, true) // storage(write) allows reading as well 
        | (Some(StorageOperation::ReadsWrites), true, true) => (),
    };

    (reads, writes)
}
