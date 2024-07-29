//! Value numbering based common subexpression elimination

use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHasher};
use slotmap::Key;
use std::{
    collections::hash_map,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use crate::{
    block, function_print, AnalysisResults, BinaryOpKind, Context, DebugWithContext, Function,
    InstOp, IrError, Pass, PassMutability, PostOrder, Predicate, ScopedPass, Type, UnaryOpKind,
    Value, POSTORDER_NAME,
};

pub const CSE_NAME: &str = "cse";

pub fn create_cse_pass() -> Pass {
    Pass {
        name: CSE_NAME,
        descr: "Common subexpression elimination",
        runner: ScopedPass::FunctionPass(PassMutability::Transform(cse)),
        deps: vec![POSTORDER_NAME],
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
        InstOp::GetConfig(_, _) => None,
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
        InstOp::MemCopyBytes { .. } => None,
        InstOp::MemCopyVal { .. } => None,
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
        write!(f, "value_map:\n")?;
        for (key, value) in &self.value_map {
            write!(f, "\tv{:?} -> {:?}\n", key.0.data(), value)?
        }
        Ok(())
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

    // Initialize all instructions and constants. Constants need special treatmemt.
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
                    const_opd_const.eq(context, c)
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
                                            (vn1 == vn2).then(|| ValueNumber::Number(vn1))
                                        }
                                    }
                                } else {
                                    None
                                }
                            })
                            .flatten()
                            // The PHI couldn't be simplifed to a single ValueNumber.
                            // lookup(expr, x)
                            .unwrap_or_else(|| match vntable.expr_map.entry(expr) {
                                hash_map::Entry::Occupied(occ) => *occ.get(),
                                hash_map::Entry::Vacant(vac) => {
                                    *(vac.insert(ValueNumber::Number(phi)))
                                }
                            })
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

    Ok(false)
}
