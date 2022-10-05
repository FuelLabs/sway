use crate::{
    asm_generation::{register_allocator, AllocatedAbstractInstructionSet, RegisterSequencer},
    asm_lang::{
        allocated_ops::AllocatedOp, AllocatedAbstractOp, Op, OrganizationalOp, RealizedOp,
        VirtualOp, VirtualRegister,
    },
};

use std::{collections::BTreeSet, fmt};

use either::Either;

/// An [AbstractInstructionSet] is a set of instructions that use entirely virtual registers
/// and excessive moves, with the intention of later optimizing it.
#[derive(Clone)]
pub struct AbstractInstructionSet {
    pub(crate) ops: Vec<Op>,
}

impl AbstractInstructionSet {
    pub(crate) fn optimize(self) -> AbstractInstructionSet {
        self.remove_sequential_jumps()
            .remove_redundant_moves()
            .remove_unused_ops()
    }

    /// Removes any jumps that jump to the subsequent line
    fn remove_sequential_jumps(mut self) -> AbstractInstructionSet {
        let dead_jumps: Vec<_> = self
            .ops
            .windows(2)
            .enumerate()
            .filter_map(|(idx, ops)| match (&ops[0].opcode, &ops[1].opcode) {
                (
                    Either::Right(OrganizationalOp::Jump(dst_label)),
                    Either::Right(OrganizationalOp::Label(label)),
                ) if dst_label == label => Some(idx),
                _otherwise => None,
            })
            .collect();

        // Replace the dead jumps with NOPs, as it's cheaper.
        for idx in dead_jumps {
            self.ops[idx] = Op {
                opcode: Either::Left(VirtualOp::NOOP),
                comment: "removed redundant JUMP".into(),
                owning_span: None,
            };
        }

        self
    }

    fn remove_redundant_moves(mut self) -> AbstractInstructionSet {
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
            let uses: BTreeSet<&VirtualRegister> =
                self.ops.iter().fold(BTreeSet::new(), |mut acc, op| {
                    acc.append(&mut op.use_registers());
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
                    comment: "removed redundant MOVE".into(),
                    owning_span: None,
                };
            }
        }

        self
    }

    fn remove_unused_ops(mut self) -> AbstractInstructionSet {
        // Just remove NOPs for now.
        self.ops.retain(|op| match &op.opcode {
            Either::Left(VirtualOp::NOOP) => false,
            _otherwise => true,
        });

        self
    }

    /// Assigns an allocatable register to each virtual register used by some instruction in the
    /// list `self.ops`. The algorithm used is Chaitin's graph-coloring register allocation
    /// algorithm (https://en.wikipedia.org/wiki/Chaitin%27s_algorithm). The individual steps of
    /// the algorithm are thoroughly explained in register_allocator.rs.
    ///
    pub(crate) fn allocate_registers(
        self,
        register_sequencer: &mut RegisterSequencer,
    ) -> AllocatedAbstractInstructionSet {
        // Step 1: Liveness Analysis.
        let live_out = register_allocator::liveness_analysis(&self.ops);

        // Step 2: Construct the interference graph.
        let (mut interference_graph, mut reg_to_node_ix) =
            register_allocator::create_interference_graph(&self.ops, &live_out);

        // Step 3: Remove redundant MOVE instructions using the interference graph.
        let reduced_ops = register_allocator::coalesce_registers(
            &self.ops,
            &mut interference_graph,
            &mut reg_to_node_ix,
            register_sequencer,
        );

        // Step 4: Simplify - i.e. color the interference graph and return a stack that contains
        // each colorable node and its neighbors.
        let mut stack = register_allocator::color_interference_graph(&mut interference_graph);

        // Step 5: Use the stack to assign a register for each virtual register.
        let pool = register_allocator::assign_registers(&mut stack);

        // Step 6: Update all instructions to use the resulting register pool.
        let mut buf = vec![];
        for op in &reduced_ops {
            buf.push(AllocatedAbstractOp {
                opcode: op.allocate_registers(&pool),
                comment: op.comment.clone(),
                owning_span: op.owning_span.clone(),
            })
        }

        AllocatedAbstractInstructionSet { ops: buf }
    }
}

impl fmt::Display for AbstractInstructionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".program:\n{}",
            self.ops
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// "Realized" here refers to labels -- there are no more organizational
/// ops or labels. In this struct, they are all "realized" to offsets.
pub struct RealizedAbstractInstructionSet {
    pub(super) ops: Vec<RealizedOp>,
}

impl RealizedAbstractInstructionSet {
    pub(crate) fn pad_to_even(self) -> Vec<AllocatedOp> {
        let mut ops = self
            .ops
            .into_iter()
            .map(
                |RealizedOp {
                     opcode,
                     comment,
                     owning_span,
                     offset: _,
                 }| {
                    AllocatedOp {
                        opcode,
                        comment,
                        owning_span,
                    }
                },
            )
            .collect::<Vec<_>>();

        if ops.len() & 1 != 0 {
            ops.push(AllocatedOp {
                opcode: crate::asm_lang::allocated_ops::AllocatedOpcode::NOOP,
                comment: "word-alignment of data section".into(),
                owning_span: None,
            });
        }

        ops
    }
}
