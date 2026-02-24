use std::collections::{BTreeSet, HashMap};

use either::Either;
use rustc_hash::FxHashSet;

use crate::asm_lang::{ControlFlowOp, JumpType, Label};

use super::super::{abstract_instruction_set::AbstractInstructionSet, analyses::liveness_analysis};

impl AbstractInstructionSet {
    pub(crate) fn dce(mut self) -> AbstractInstructionSet {
        let liveness = liveness_analysis(&self.ops, false);
        let ops = &self.ops;

        let mut cur_live = BTreeSet::default();
        let mut dead_indices = FxHashSet::default();
        for (rev_ix, op) in ops.iter().rev().enumerate() {
            // We cannot guarantee the jump will not end in a 
            // instruction that will be eliminated below
            if let Either::Right(ControlFlowOp::JumpToAddr(..)) = &op.opcode {
                return self;
            }

            let ix = ops.len() - rev_ix - 1;

            let op_use = op.use_registers();
            let mut op_def = op.def_registers();
            op_def.append(&mut op.def_const_registers());

            if let Either::Right(ControlFlowOp::Jump { type_, .. }) = &op.opcode {
                if !matches!(type_, JumpType::Call) {
                    // Block boundary. Start afresh.
                    cur_live.clone_from(liveness.get(ix).expect("Incorrect liveness info"));
                    // Add use(op) to cur_live.
                    for u in op_use {
                        cur_live.insert(u.clone());
                    }
                    continue;
                }
            }

            let dead = op_def.iter().all(|def| !cur_live.contains(def))
                && match &op.opcode {
                    Either::Left(op) => !op.has_side_effect(),
                    Either::Right(_) => false,
                };
            // Remove def(op) from cur_live.
            for def in &op_def {
                cur_live.remove(def);
            }
            if dead {
                dead_indices.insert(ix);
            } else {
                // Add use(op) to cur_live
                for u in op_use {
                    cur_live.insert(u.clone());
                }
            }
        }

        // Actually delete the instructions.
        let mut new_ops: Vec<_> = std::mem::take(&mut self.ops)
            .into_iter()
            .enumerate()
            .filter_map(|(idx, op)| {
                if !dead_indices.contains(&idx) {
                    Some(op)
                } else {
                    None
                }
            })
            .collect();
        std::mem::swap(&mut self.ops, &mut new_ops);

        self
    }

    // Remove unreachable instructions.
    pub(crate) fn simplify_cfg(mut self) -> AbstractInstructionSet {
        let ops = &self.ops;

        if ops.is_empty() {
            return self;
        }

        // Keep track of a map between jump labels and op indices. Useful to compute op successors.
        let mut label_to_index: HashMap<Label, usize> = HashMap::default();
        for (idx, op) in ops.iter().enumerate() {
            if let Either::Right(ControlFlowOp::Label(op_label)) = op.opcode {
                label_to_index.insert(op_label, idx);
            }
        }

        let mut reachables = vec![false; ops.len()];
        let mut worklist = vec![0];
        while let Some(op_idx) = worklist.pop() {
            if reachables[op_idx] {
                continue;
            }
            reachables[op_idx] = true;
            let op = &ops[op_idx];
            for s in &op.successors(op_idx, ops, &label_to_index) {
                if reachables[*s] {
                    continue;
                }
                worklist.push(*s);
            }
        }

        let reachable_ops = self
            .ops
            .into_iter()
            .enumerate()
            .filter_map(|(idx, op)| if reachables[idx] { Some(op) } else { None })
            .collect();
        self.ops = reachable_ops;

        self
    }
}
