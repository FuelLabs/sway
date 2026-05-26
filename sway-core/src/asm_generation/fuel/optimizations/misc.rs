use super::super::abstract_instruction_set::AbstractInstructionSet;

use crate::asm_lang::{
    virtual_register::ConstantRegister, JumpType, Op, OrganizationalOp, VirtualOp, VirtualRegister,
};

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

            // Replace the dead moves with NOOPs, as it's cheaper.
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

    pub(crate) fn remove_redundant_ops(mut self, mut log: impl FnMut(&str),) -> AbstractInstructionSet {
        let mut new_ops = Vec::with_capacity(self.ops.len());

        let mut ops = self.ops.iter().peekable();
        while let Some(op) = ops.next() {
            let remove = match &op.opcode {
                Either::Left(VirtualOp::NOOP) => true,
                Either::Left(VirtualOp::MOVE(a, b)) => a == b,
                Either::Left(VirtualOp::MCP(_, _, len)) => {
                    matches!(len, VirtualRegister::Constant(ConstantRegister::Zero))
                }
                Either::Left(VirtualOp::MCPI(_, _, imm)) => imm.value() == 0,
                _ => false,
            };

            // We also need to be sure op is redundant regarding const registers.
            let remove = if remove {
                if let Some(next_op) = ops.peek() {
                    op.def_const_registers().intersection(&next_op.use_registers()).count() == 0
                } else {
                    // last instruction, we can remove it
                    true
                }
            } else {
                false
            };

            if !remove {
                log(&format!("    keeping: {}\n", op));
                new_ops.push(op.clone())
            } else {
                log(&format!("    removing: {}\n", op));
            }
        }

        self.ops = new_ops;
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn remove_redundant_ops() {
        let mut str = String::new();
        let capture = |s: &str| {
            str.push_str(s);
        };
        super::super::constant_propagate::tests::optimise(
            [
                // NOOP
                VirtualOp::noop().into(),
                VirtualOp::noop().into(),
                VirtualOp::r#move("0", "err").into(),
                VirtualOp::noop().into(),
                VirtualOp::r#move("0", "of").into(),
                // MOVE with same registers
                VirtualOp::r#move("0", "0").into(),
                VirtualOp::r#move("1", "1").into(),
                VirtualOp::r#move("0", "err").into(),
                VirtualOp::r#move("2", "2").into(),
                VirtualOp::r#move("0", "of").into(),
            ],
            |ops| ops.remove_redundant_ops(capture),
        );

        expect![[r#"
                removing: noop                                    ; 0
                keeping: noop                                    ; 1
                keeping: move $r0 $err                           ; 2
                keeping: noop                                    ; 3
                keeping: move $r0 $of                            ; 4
                removing: move $r0 $r0                            ; 5
                keeping: move $r1 $r1                            ; 6
                keeping: move $r0 $err                           ; 7
                keeping: move $r2 $r2                            ; 8
                keeping: move $r0 $of                            ; 9
        "#]]
        .assert_eq(&str);
    }
}