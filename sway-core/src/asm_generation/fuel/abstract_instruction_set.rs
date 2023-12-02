use crate::{
    asm_generation::fuel::{
        allocated_abstract_instruction_set::AllocatedAbstractInstructionSet, register_allocator,
    },
    asm_lang::{
        allocated_ops::{AllocatedOp, AllocatedOpcode},
        Op, OrganizationalOp, RealizedOp, VirtualOp, VirtualRegister,
    },
};

use sway_error::error::CompileError;
use sway_types::Span;

use std::{collections::HashSet, fmt};

use either::Either;

use super::data_section::DataSection;

/// An [AbstractInstructionSet] is a set of instructions that use entirely virtual registers
/// and excessive moves, with the intention of later optimizing it.
#[derive(Clone)]
pub struct AbstractInstructionSet {
    pub(crate) ops: Vec<Op>,
}

impl AbstractInstructionSet {
    pub(crate) fn optimize(self, data_section: &DataSection) -> AbstractInstructionSet {
        self.const_indexing_aggregates_function(data_section)
            .dce()
            .remove_sequential_jumps()
            .remove_redundant_moves()
            .remove_unused_ops()
    }

    /// Removes any jumps to the subsequent line.
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

    pub(crate) fn verify(self) -> Result<AbstractInstructionSet, CompileError> {
        // At the moment the only verification we do is to make sure used registers are
        // initialised.  Without doing dataflow analysis we still can't guarantee the init is
        // _before_ the use, but future refactoring to convert abstract ops into SSA and BBs will
        // make this possible or even make this check redundant.

        macro_rules! add_virt_regs {
            ($regs: expr, $set: expr) => {
                let mut regs = $regs;
                regs.retain(|&reg| matches!(reg, VirtualRegister::Virtual(_)));
                $set.extend(regs.into_iter());
            };
        }

        let mut use_regs = HashSet::new();
        let mut def_regs = HashSet::new();
        for op in &self.ops {
            add_virt_regs!(op.use_registers(), use_regs);
            add_virt_regs!(op.def_registers(), def_regs);
        }

        if def_regs.is_superset(&use_regs) {
            Ok(self)
        } else {
            let bad_regs = use_regs
                .difference(&def_regs)
                .map(|reg| match reg {
                    VirtualRegister::Virtual(name) => format!("$r{name}"),
                    VirtualRegister::Constant(creg) => creg.to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            Err(CompileError::InternalOwned(
                format!("Program erroneously uses uninitialized virtual registers: {bad_regs}"),
                Span::dummy(),
            ))
        }
    }

    /// Allocate registers.
    pub(crate) fn allocate_registers(
        self,
    ) -> Result<AllocatedAbstractInstructionSet, CompileError> {
        register_allocator::allocate_registers(&self.ops)
    }
}

impl fmt::Display for AbstractInstructionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".program:\n{}",
            self.ops
                .iter()
                .map(|x| format!("{x}"))
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
                opcode: AllocatedOpcode::NOOP,
                comment: "word-alignment of data section".into(),
                owning_span: None,
            });
        }

        ops
    }
}
