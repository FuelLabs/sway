use crate::{
    language::{
        promote_purity,
        Purity::{self, *},
    },
    metadata::{MetadataManager, StorageOperation},
};

use sway_error::{
    error::CompileError,
    handler::Handler,
    warning::{CompileWarning, Warning},
};
use sway_ir::{Context, Function, Instruction};
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
                    match instruction {
                        Instruction::StateLoadQuadWord { .. } | Instruction::StateLoadWord(_) => {
                            (true, writes)
                        }

                        Instruction::StateStoreQuadWord { .. }
                        | Instruction::StateStoreWord { .. } => (reads, true),

                        // Iterate for and check each instruction in the ASM block.
                        Instruction::AsmBlock(asm_block, _args) => {
                            asm_block.get_content(context).body.iter().fold(
                                (reads, writes),
                                |(reads, writes), asm_op| match asm_op.name.as_str() {
                                    "srw" | "srwq" => (true, writes),
                                    "sww" | "swwq" => (reads, true),
                                    _ => (reads, writes),
                                },
                            )
                        }

                        // Recurse to find the called function purity.  Use memoisation to
                        // avoid redoing work.
                        Instruction::Call(callee, _args) => {
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
        handler.emit_warn(CompileWarning {
            warning_content: Warning::DeadStorageDeclarationForFunction {
                unneeded_attrib: purity.to_attribute_syntax(),
            },
            span,
        });
    };

    match (attributed_purity, reads, writes) {
        // Has no attributes but needs some.
        (None, true, false) => error(span, "read", Pure, Reads),
        (None, false, true) => error(span, "write", Pure, Writes),
        (None, true, true) => error(span, "read & write", Pure, ReadsWrites),

        // Or the attribute must match the behaviour.
        (Some(StorageOperation::Reads), _, true) => error(span, "write", Reads, Writes),
        (Some(StorageOperation::Writes), true, _) => error(span, "read", Writes, Reads),

        // Or we have unneeded attributes.
        (Some(StorageOperation::ReadsWrites), false, true) => warn(span, Reads),
        (Some(StorageOperation::ReadsWrites), true, false) => warn(span, Writes),
        (Some(StorageOperation::Reads), false, false) => warn(span, Reads),
        (Some(StorageOperation::Writes), false, false) => warn(span, Writes),

        // (Pure, false, false) is OK, as is (ReadsWrites, true, true).
        _ => (),
    };

    (reads, writes)
}
