//! Value numbering based common subexpression elimination.
//! Reference: Value Driven Redundancy Elimination - Loren Taylor Simpson.

use core::panic;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use slotmap::Key;
use std::{
    collections::hash_map,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use crate::{
    AnalysisResults, BinaryOpKind, Context, DebugWithContext, DomTree, Function, InstOp, IrError,
    Pass, PassMutability, PostOrder, Predicate, ScopedPass, Type, UnaryOpKind, Value,
    DOMINATORS_NAME, POSTORDER_NAME,
};

pub const CSE_NAME: &str = "cse";

pub fn create_cse_pass() -> Pass {
    Pass {
        name: CSE_NAME,
        descr: "Common subexpression elimination",
        runner: ScopedPass::FunctionPass(PassMutability::Transform(cse)),
        deps: vec![POSTORDER_NAME, DOMINATORS_NAME],
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, DebugWithContext)]
enum ValueNumber {
    // Top of the lattice = Don't know = uninitialized
    Top,
    // Belongs to a congruence class represented by the inner value.
    Number(Value),
}

impl Debug for ValueNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Top => write!(f, "Top"),
            Self::Number(arg0) => write!(f, "v{:?}", arg0.0.data()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, DebugWithContext)]
enum Expr {
    Phi(Vec<ValueNumber>),
    UnaryOp {
        op: UnaryOpKind,
        arg: ValueNumber,
    },
    BinaryOp {
        op: BinaryOpKind,
        arg1: ValueNumber,
        arg2: ValueNumber,
    },
    BitCast(ValueNumber, Type),
    CastPtr(ValueNumber, Type),
    Cmp(Predicate, ValueNumber, ValueNumber),
    GetElemPtr {
        base: ValueNumber,
        elem_ptr_ty: Type,
        indices: Vec<ValueNumber>,
    },
    IntToPtr(ValueNumber, Type),
    PtrToInt(ValueNumber, Type),
}

/// Convert an instruction to an expression for hashing
/// Instructions that we don't handle will have their value numbers be equal to themselves.
fn instr_to_expr(context: &Context, vntable: &VNTable, instr: Value) -> Option<Expr> {
    match &instr.get_instruction(context).unwrap().op {
        InstOp::AsmBlock(_, _) => None,
        InstOp::UnaryOp { op, arg } => Some(Expr::UnaryOp {
            op: *op,
            arg: vntable.value_map.get(arg).cloned().unwrap(),
        }),
        InstOp::BinaryOp { op, arg1, arg2 } => Some(Expr::BinaryOp {
            op: *op,
            arg1: vntable.value_map.get(arg1).cloned().unwrap(),
            arg2: vntable.value_map.get(arg2).cloned().unwrap(),
        }),
        InstOp::BitCast(val, ty) => Some(Expr::BitCast(
            vntable.value_map.get(val).cloned().unwrap(),
            *ty,
        )),
        InstOp::Branch(_) => None,
        InstOp::Call(_, _) => None,
        InstOp::CastPtr(val, ty) => Some(Expr::CastPtr(
            vntable.value_map.get(val).cloned().unwrap(),
            *ty,
        )),
        InstOp::Cmp(pred, val1, val2) => Some(Expr::Cmp(
            *pred,
            vntable.value_map.get(val1).cloned().unwrap(),
            vntable.value_map.get(val2).cloned().unwrap(),
        )),
        InstOp::ConditionalBranch { .. } => None,
        InstOp::ContractCall { .. } => None,
        InstOp::FuelVm(_) => None,
        InstOp::GetLocal(_) => None,
        InstOp::GetGlobal(_) => None,
        InstOp::GetConfig(_, _) => None,
        InstOp::GetStorageKey(_) => None,
        InstOp::GetElemPtr {
            base,
            elem_ptr_ty,
            indices,
        } => Some(Expr::GetElemPtr {
            base: vntable.value_map.get(base).cloned().unwrap(),
            elem_ptr_ty: *elem_ptr_ty,
            indices: indices
                .iter()
                .map(|idx| vntable.value_map.get(idx).cloned().unwrap())
                .collect(),
        }),
        InstOp::IntToPtr(val, ty) => Some(Expr::IntToPtr(
            vntable.value_map.get(val).cloned().unwrap(),
            *ty,
        )),
        InstOp::Load(_) => None,
        InstOp::Alloc { .. } => None,
        InstOp::MemCopyBytes { .. } => None,
        InstOp::MemCopyVal { .. } => None,
        InstOp::MemClearVal { .. } => None,
        InstOp::Nop => None,
        InstOp::PtrToInt(val, ty) => Some(Expr::PtrToInt(
            vntable.value_map.get(val).cloned().unwrap(),
            *ty,
        )),
        InstOp::Ret(_, _) => None,
        InstOp::Store { .. } => None,
    }
}

/// Convert a PHI argument to Expr
fn phi_to_expr(context: &Context, vntable: &VNTable, phi_arg: Value) -> Expr {
    let phi_arg = phi_arg.get_argument(context).unwrap();
    let phi_args = phi_arg
        .block
        .pred_iter(context)
        .map(|pred| {
            let incoming_val = phi_arg
                .get_val_coming_from(context, pred)
                .expect("No parameter from predecessor");
            vntable.value_map.get(&incoming_val).cloned().unwrap()
        })
        .collect();
    Expr::Phi(phi_args)
}

#[derive(Default)]
struct VNTable {
    value_map: FxHashMap<Value, ValueNumber>,
    expr_map: FxHashMap<Expr, ValueNumber>,
}

impl Debug for VNTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "value_map:")?;
        self.value_map.iter().for_each(|(key, value)| {
            if format!("v{:?}", key.0.data()) == "v620v3" {
                writeln!(f, "\tv{:?} -> {:?}", key.0.data(), value).expect("writeln! failed");
            }
        });
        Ok(())
    }
}

/// Wrapper around [DomTree::dominates] to work at instruction level.
/// Does `inst1` dominate `inst2` ?
fn dominates(context: &Context, dom_tree: &DomTree, inst1: Value, inst2: Value) -> bool {
    let block1 = match &context.values[inst1.0].value {
        crate::ValueDatum::Argument(arg) => arg.block,
        crate::ValueDatum::Constant(_) => {
            panic!("Shouldn't be querying dominance info for constants")
        }
        crate::ValueDatum::Instruction(i) => i.parent,
    };
    let block2 = match &context.values[inst2.0].value {
        crate::ValueDatum::Argument(arg) => arg.block,
        crate::ValueDatum::Constant(_) => {
            panic!("Shouldn't be querying dominance info for constants")
        }
        crate::ValueDatum::Instruction(i) => i.parent,
    };

    if block1 == block2 {
        let inst1_idx = block1
            .instruction_iter(context)
            .position(|inst| inst == inst1)
            // Not found indicates a block argument
            .unwrap_or(0);
        let inst2_idx = block1
            .instruction_iter(context)
            .position(|inst| inst == inst2)
            // Not found indicates a block argument
            .unwrap_or(0);
        inst1_idx < inst2_idx
    } else {
        dom_tree.dominates(block1, block2)
    }
}

pub fn cse(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let mut vntable = VNTable::default();

    // Function arg values map to themselves.
    for arg in function.args_iter(context) {
        vntable.value_map.insert(arg.1, ValueNumber::Number(arg.1));
    }

    // Map all other arg values map to Top.
    for block in function.block_iter(context).skip(1) {
        for arg in block.arg_iter(context) {
            vntable.value_map.insert(*arg, ValueNumber::Top);
        }
    }

    // Initialize all instructions and constants. Constants need special treatment.
    // They don't have PartialEq implemented. So we need to value number them manually.
    // This map maps the hash of a constant value to all possible collisions of it.
    let mut const_map = FxHashMap::<u64, Vec<Value>>::default();
    for (_, inst) in function.instruction_iter(context) {
        vntable.value_map.insert(inst, ValueNumber::Top);
        for (const_opd_val, const_opd_const) in inst
            .get_instruction(context)
            .unwrap()
            .op
            .get_operands()
            .iter()
            .filter_map(|opd| opd.get_constant(context).map(|copd| (opd, copd)))
        {
            let mut state = FxHasher::default();
            const_opd_const.hash(&mut state);
            let hash = state.finish();
            if let Some(existing_const) = const_map.get(&hash).and_then(|consts| {
                consts.iter().find(|val| {
                    let c = val
                        .get_constant(context)
                        .expect("const_map can only contain consts");
                    const_opd_const == c
                })
            }) {
                vntable
                    .value_map
                    .insert(*const_opd_val, ValueNumber::Number(*existing_const));
            } else {
                const_map
                    .entry(hash)
                    .and_modify(|consts| consts.push(*const_opd_val))
                    .or_insert_with(|| vec![*const_opd_val]);
                vntable
                    .value_map
                    .insert(*const_opd_val, ValueNumber::Number(*const_opd_val));
            }
        }
    }

    // We need to iterate over the blocks in RPO.
    let post_order: &PostOrder = analyses.get_analysis_result(function);

    // RPO based value number (Sec 4.2).
    let mut changed = true;
    while changed {
        changed = false;
        // For each block in RPO:
        for (block_idx, block) in post_order.po_to_block.iter().rev().enumerate() {
            // Process PHIs and then the other instructions.
            if block_idx != 0 {
                // Entry block arguments are not PHIs.
                for (phi, expr_opt) in block
                    .arg_iter(context)
                    .map(|arg| (*arg, Some(phi_to_expr(context, &vntable, *arg))))
                    .collect_vec()
                {
                    let expr = expr_opt.expect("PHIs must always translate to a valid Expr");
                    // We first try to see if PHIs can be simplified into a single value.
                    let vn = {
                        let Expr::Phi(ref phi_args) = expr else {
                            panic!("Expr must be a PHI")
                        };
                        phi_args
                            .iter()
                            .map(|vn| Some(*vn))
                            .reduce(|vn1, vn2| {
                                // Here `None` indicates Bottom of the lattice.
                                if let (Some(vn1), Some(vn2)) = (vn1, vn2) {
                                    match (vn1, vn2) {
                                        (ValueNumber::Top, ValueNumber::Top) => {
                                            Some(ValueNumber::Top)
                                        }
                                        (ValueNumber::Top, ValueNumber::Number(vn))
                                        | (ValueNumber::Number(vn), ValueNumber::Top) => {
                                            Some(ValueNumber::Number(vn))
                                        }
                                        (ValueNumber::Number(vn1), ValueNumber::Number(vn2)) => {
                                            (vn1 == vn2).then_some(ValueNumber::Number(vn1))
                                        }
                                    }
                                } else {
                                    None
                                }
                            })
                            .flatten()
                            // The PHI couldn't be simplified to a single ValueNumber.
                            .unwrap_or(ValueNumber::Number(phi))
                    };

                    match vntable.value_map.entry(phi) {
                        hash_map::Entry::Occupied(occ) if *occ.get() == vn => {}
                        _ => {
                            changed = true;
                            vntable.value_map.insert(phi, vn);
                        }
                    }
                }
            }

            for (inst, expr_opt) in block
                .instruction_iter(context)
                .map(|instr| (instr, instr_to_expr(context, &vntable, instr)))
                .collect_vec()
            {
                // lookup(expr, x)
                let vn = if let Some(expr) = expr_opt {
                    match vntable.expr_map.entry(expr) {
                        hash_map::Entry::Occupied(occ) => *occ.get(),
                        hash_map::Entry::Vacant(vac) => *(vac.insert(ValueNumber::Number(inst))),
                    }
                } else {
                    // Instructions that always map to their own value number
                    // (i.e., they can never be equal to some other instruction).
                    ValueNumber::Number(inst)
                };
                match vntable.value_map.entry(inst) {
                    hash_map::Entry::Occupied(occ) if *occ.get() == vn => {}
                    _ => {
                        changed = true;
                        vntable.value_map.insert(inst, vn);
                    }
                }
            }
        }
        vntable.expr_map.clear();
    }

    // create a partition of congruent (equal) values.
    let mut partition = FxHashMap::<ValueNumber, FxHashSet<Value>>::default();
    vntable.value_map.iter().for_each(|(v, vn)| {
        // If v is a constant or its value number is itself, don't add to the partition.
        // The latter condition is so that we have only > 1 sized partitions.
        if v.is_constant(context)
            || matches!(vn, ValueNumber::Top)
            || matches!(vn, ValueNumber::Number(v2) if (v == v2 || v2.is_constant(context)))
        {
            return;
        }
        partition
            .entry(*vn)
            .and_modify(|part| {
                part.insert(*v);
            })
            .or_insert(vec![*v].into_iter().collect());
    });

    // For convenience, now add back back `v` into `partition[VN[v]]` if it isn't already there.
    partition.iter_mut().for_each(|(vn, v_part)| {
        let ValueNumber::Number(v) = vn else {
            panic!("We cannot have Top at this point");
        };
        v_part.insert(*v);
        assert!(
            v_part.len() > 1,
            "We've only created partitions with size greater than 1"
        );
    });

    // There are two ways to replace congruent values (see the paper cited, Sec 5).
    // 1. Dominator based. If v1 and v2 are equal, v1 dominates v2, we just remove v2
    // and replace its uses with v1. Simple, and what we're going to do.
    // 2. AVAIL based. More powerful, but also requires a data-flow-analysis for AVAIL
    // and later on, mem2reg again since replacements will need breaking SSA.
    let dom_tree: &DomTree = analyses.get_analysis_result(function);
    let mut replace_map = FxHashMap::<Value, Value>::default();
    let mut modified = false;
    // Check every set in the partition.
    partition.iter().for_each(|(_leader, vals)| {
        // Iterate over every pair of values, checking if one can replace the other.
        for v_pair in vals.iter().combinations(2) {
            let (v1, v2) = (*v_pair[0], *v_pair[1]);
            if dominates(context, dom_tree, v1, v2) {
                modified = true;
                replace_map.insert(v2, v1);
            } else if dominates(context, dom_tree, v2, v1) {
                modified = true;
                replace_map.insert(v1, v2);
            }
        }
    });

    function.replace_values(context, &replace_map, None);

    Ok(modified)
}
