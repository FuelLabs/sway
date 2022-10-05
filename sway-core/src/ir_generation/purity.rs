use crate::{
    error::*,
    metadata::{MetadataManager, StorageOperation},
    parse_tree::{promote_purity, Purity},
};

use sway_ir::{Context, Function, Instruction};
use sway_types::span::Span;

use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PurityChecker {
    memos: HashMap<Function, (bool, bool)>,

    // Final results.
    warnings: Vec<CompileWarning>,
    errors: Vec<CompileError>,
}

impl PurityChecker {
    /// Designed to be called for each entry point, _prior_ to inlining or other optimizations.
    /// The checker will check this function and any that it calls.
    ///
    /// Returns bools for whether it (reads, writes).
    pub(crate) fn check_function(
        &mut self,
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
                            Instruction::StateLoadQuadWord { .. }
                            | Instruction::StateLoadWord(_) => (true, writes),

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
                                    self.memos.get(callee).copied().unwrap_or_else(|| {
                                        let r_w = self.check_function(context, md_mgr, callee);
                                        self.memos.insert(*callee, r_w);
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

        // Simple macros for each of the error types, which also grab `span`.
        macro_rules! mk_err {
            ($op_str:literal, $existing_attrib:ident, $needed_attrib:ident) => {{
                self.errors.push(CompileError::ImpureInPureContext {
                    storage_op: $op_str,
                    attrs: promote_purity(Purity::$existing_attrib, Purity::$needed_attrib)
                        .to_attribute_syntax(),
                    span,
                });
            }};
        }
        macro_rules! mk_warn {
            ($unneeded_attrib:ident) => {{
                self.warnings.push(CompileWarning {
                    warning_content: Warning::DeadStorageDeclarationForFunction {
                        unneeded_attrib: Purity::$unneeded_attrib.to_attribute_syntax(),
                    },
                    span,
                });
            }};
        }

        match (attributed_purity, reads, writes) {
            // Has no attributes but needs some.
            (None, true, false) => mk_err!("read", Pure, Reads),
            (None, false, true) => mk_err!("write", Pure, Writes),
            (None, true, true) => mk_err!("read & write", Pure, ReadsWrites),

            // Or the attribute must match the behaviour.
            (Some(StorageOperation::Reads), _, true) => mk_err!("write", Reads, Writes),
            (Some(StorageOperation::Writes), true, _) => mk_err!("read", Writes, Reads),

            // Or we have unneeded attributes.
            (Some(StorageOperation::ReadsWrites), false, true) => mk_warn!(Reads),
            (Some(StorageOperation::ReadsWrites), true, false) => mk_warn!(Writes),
            (Some(StorageOperation::Reads), false, false) => mk_warn!(Reads),
            (Some(StorageOperation::Writes), false, false) => mk_warn!(Writes),

            // (Pure, false, false) is OK, as is (ReadsWrites, true, true).
            _otherwise => (),
        };

        (reads, writes)
    }

    pub(crate) fn results(self) -> CompileResult<()> {
        if self.errors.is_empty() {
            ok((), self.warnings, self.errors)
        } else {
            err(self.warnings, self.errors)
        }
    }
}
