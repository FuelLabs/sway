use super::super::abstract_instruction_set::AbstractInstructionSet;

use crate::asm_lang::{JumpType, Op, OrganizationalOp, VirtualOp, VirtualRegister};

use std::collections::HashSet;

use either::Either;

impl AbstractInstructionSet {
    /// Removes any jumps to the subsequent line.
    pub(crate) fn remove_sequential_jumps(mut self) -> AbstractInstructionSet {
        let dead_jumps: Vec<_> = self
            .ops
            .windows(2)
            .enumerate()
            .filter_map(|(idx, ops)| match (&ops[0].opcode, &ops[1].opcode) {
                (
                    Either::Right(OrganizationalOp::Jump {
                        to: dst_label,
                        type_: JumpType::Unconditional | JumpType::NotZero(_),
                        ..
                    }),
                    Either::Right(OrganizationalOp::Label(label)),
                ) if dst_label == label => Some(idx),
                _otherwise => None,
            })
            .collect();

        // Replace the dead jumps with NOPs, as it's cheaper.
        for idx in dead_jumps {
            self.ops[idx] = Op {
                opcode: Either::Left(VirtualOp::NOOP),
                comment: "remove redundant jump operation".into(),
                owning_span: None,
            };
        }

        self
    }

    pub(crate) fn remove_redundant_moves(mut self) -> AbstractInstructionSet {
        // This has a lot of room for improvement.
        //
        // For now it is just removing MOVEs to registers which are _never_ used.  It doesn't
        // analyse control flow or other redundancies.  Some obvious improvements are:
        //
        // - Perform a control flow analysis to remove MOVEs to registers which are not used
        // _after_ the MOVE.
        //
        // - Remove the redundant use of temporaries.  E.g.:
        //     MOVE t, a        MOVE b, a
        //     MOVE b, t   =>   USE  b
        //     USE  b
        loop {
            // Gather all the uses for each register.
            let uses: HashSet<&VirtualRegister> =
                self.ops.iter().fold(HashSet::new(), |mut acc, op| {
                    for u in &op.use_registers() {
                        acc.insert(u);
                    }
                    acc
                });

            // Loop again and find MOVEs which have a non-constant destination which is never used.
            let mut dead_moves = Vec::new();
            for (idx, op) in self.ops.iter().enumerate() {
                if let Either::Left(VirtualOp::MOVE(
                    dst_reg @ VirtualRegister::Virtual(_),
                    _src_reg,
                )) = &op.opcode
                {
                    if !uses.contains(dst_reg) {
                        dead_moves.push(idx);
                    }
                }
            }

            if dead_moves.is_empty() {
                break;
            }

            // Replace the dead moves with NOPs, as it's cheaper.
            for idx in dead_moves {
                self.ops[idx] = Op {
                    opcode: Either::Left(VirtualOp::NOOP),
                    comment: "remove redundant move operation".into(),
                    owning_span: None,
                };
            }
        }

        self
    }

    pub(crate) fn remove_redundant_ops(mut self) -> AbstractInstructionSet {
        self.ops.retain(|op| {
            // It is easier to think in terms of operations we want to remove
            // than the operations we want to retain ;-)
            #[allow(clippy::match_like_matches_macro)]
            // Keep the `match` for adding more ops in the future.
            let remove = match &op.opcode {
                Either::Left(VirtualOp::NOOP) => true,
                Either::Left(VirtualOp::MOVE(a, b)) => a == b,
                Either::Left(VirtualOp::CFEI(_, imm)) | Either::Left(VirtualOp::CFSI(_, imm)) => {
                    imm.value() == 0
                }
                _ => false,
            };

            !remove
        });

        self
    }
}
