use std::collections::{BTreeSet, HashMap};

use either::Either;
use indexmap::IndexSet;
use sway_types::FxIndexSet;

use crate::asm_lang::{ControlFlowOp, Label, Op, VirtualRegister};

/// Given a list of instructions `ops` of a program, do liveness analysis for the full program.
///
/// A virtual registers is live at some point in the program if it has previously been defined by
/// an instruction and will be used by an instruction in the future.
///
/// The analysis function below assumes that it is possible that a virtual register is assigned
/// more than once. That is, it doesn't assume that the intermediate assembly is in SSA form.
///
/// Two tables are generated: `live_in` and `live_out`. Each row in the tables corresponds to an
/// instruction in the program.
/// * A virtual register is in the `live_out` table for a given instruction if it is live on any
///   of that node's out-edges
/// * A virtual register is in the `live_in` table for a given instruction if it is live on any
///   of that node's in-edges
///
///
/// Algorithm:
/// ===============================================================================================
/// for each instruction op:
///     live_in(op) = {}
///     live_out(op) = {}
///     def(op) = list of virtual registers defined by op
///     use(op) = list of virtual registers used by op
///
/// repeat
///     for each instruction op (traversed in reverse topological order of the CFG)
///         prev_live_in(op) = live_in(op)
///         prev_live_out(op) = live_out(op)
///         live_out(op) = live_in(s_1) UNION live_in(s_2) UNION live_in(s_3) UNION ...
///                        where s_1, s_2, s_3, ... are all the successors of op in the CFG.
///         live_in(op) = use(op) UNION (live_out(op) - def(op))
/// until     prev_live_in(op) = live_in(op)
///       AND prev_live_out(op) = live_out(op)
/// ===============================================================================================
///
/// If `ignore_constant_regs == true` then we only look at registers that have the enum variant
/// VirtualRegister::Virtual(_). All other registers (i.e. ones with the
/// VirtualRegister::Constant(_) variant) are assumed to be live throughout the full program.
///
/// This function finally returns `live_out` because it has all the liveness information needed.
/// `live_in` is computed because it is needed to compute `live_out` iteratively.
///
pub(crate) fn liveness_analysis(
    ops: &[Op],
    ignore_constant_regs: bool,
) -> Vec<BTreeSet<VirtualRegister>> {
    // Vectors representing maps that will represent the live_in and live_out tables. Each entry
    // corresponds to an instruction in `ops`.
    let mut live_in: Vec<FxIndexSet<VirtualRegister>> = vec![IndexSet::default(); ops.len()];
    let mut live_out: Vec<BTreeSet<VirtualRegister>> = vec![BTreeSet::default(); ops.len()];
    let mut label_to_index: HashMap<Label, usize> = HashMap::new();

    // Keep track of a map between jump labels and op indices. Useful to compute op successors.
    for (idx, op) in ops.iter().enumerate() {
        if let Either::Right(ControlFlowOp::Label(op_label)) = op.opcode {
            label_to_index.insert(op_label, idx);
        }
    }

    let mut modified = true;
    while modified {
        modified = false;
        // Iterate in reverse topological order of the CFG (which is basically the same as the
        // reverse order of `ops`. This makes the outer `while` loop converge faster.
        for (ix, op) in ops.iter().rev().enumerate() {
            let mut local_modified = false;
            let rev_ix = ops.len() - ix - 1;

            // Get use and def vectors without any of the Constant registers
            let mut op_use = op.use_registers();
            let mut op_def = op.def_registers();
            if ignore_constant_regs {
                op_use.retain(|&reg| reg.is_virtual());
                op_def.retain(|&reg| reg.is_virtual());
            }

            // Compute live_out(op) = live_in(s_1) UNION live_in(s_2) UNION ..., where s1, s_2, ...
            // are successors of op
            for s in &op.successors(rev_ix, ops, &label_to_index) {
                for l in live_in[*s].iter() {
                    local_modified |= live_out[rev_ix].insert(l.clone());
                }
            }

            // Compute live_in(op) = use(op) UNION (live_out(op) - def(op))
            // Add use(op)
            for u in op_use {
                local_modified |= live_in[rev_ix].insert(u.clone());
            }
            // Add live_out(op) - def(op)
            for l in live_out[rev_ix].iter() {
                if !op_def.contains(&l) {
                    local_modified |= live_in[rev_ix].insert(l.clone());
                }
            }

            // Did anything change in this iteration?
            modified |= local_modified;
        }
    }

    live_out
}
