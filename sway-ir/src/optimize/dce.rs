//! ## Dead Code Elimination
//!
//! This optimization removes unused definitions. The pass is a combination of
//!   1. A liveness analysis that keeps track of the uses of a definition,
//!   2. At the time of inspecting a definition, if it has no uses, it is removed.
//! This pass does not do CFG transformations. That is handled by simplify_cfg.

use crate::{
    Block, BranchToWithArgs, Context, Function, Instruction, IrError, Module, Value, ValueDatum,
};

use std::collections::{HashMap, HashSet};

fn can_eliminate_instruction(context: &Context, val: Value) -> bool {
    let inst = val.get_instruction(context).unwrap();
    !inst.is_terminator() && !inst.may_have_side_effect()
}

/// Perform dead code (if any) elimination and return true if function modified.
pub fn dce(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    // Number of uses that an instruction has.
    let mut num_uses: HashMap<Value, (Block, u32)> = HashMap::new();

    fn get_operands(inst: &Instruction) -> Vec<Value> {
        match inst {
            Instruction::AddrOf(v) => vec![*v],
            Instruction::AsmBlock(_, args) => args.iter().filter_map(|aa| aa.initializer).collect(),
            Instruction::BitCast(v, _) => vec![*v],
            Instruction::BinaryOp { op: _, arg1, arg2 } => vec![*arg1, *arg2],
            Instruction::Branch(BranchToWithArgs { args, .. }) => args.clone(),
            Instruction::Call(_, vs) => vs.clone(),
            Instruction::Cmp(_, lhs, rhs) => vec![*lhs, *rhs],
            Instruction::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => {
                let mut v = vec![*cond_value];
                v.extend_from_slice(&true_block.args);
                v.extend_from_slice(&false_block.args);
                v
            }
            Instruction::ContractCall {
                return_type: _,
                name: _,
                params,
                coins,
                asset_id,
                gas,
            } => vec![*params, *coins, *asset_id, *gas],
            Instruction::ExtractElement {
                array,
                ty: _,
                index_val,
            } => vec![*array, *index_val],
            Instruction::ExtractValue {
                aggregate,
                ty: _,
                indices: _,
            } => vec![*aggregate],
            Instruction::GetStorageKey => vec![],
            Instruction::Gtf {
                index,
                tx_field_id: _,
            } => vec![*index],
            Instruction::GetPointer {
                base_ptr: _,
                ptr_ty: _,
                offset: _,
            } =>
            // TODO: Not sure.
            {
                vec![]
            }
            Instruction::InsertElement {
                array,
                ty: _,
                value,
                index_val,
            } => vec![*array, *value, *index_val],
            Instruction::InsertValue {
                aggregate,
                ty: _,
                value,
                indices: _,
            } => vec![*aggregate, *value],
            Instruction::IntToPtr(v, _) => vec![*v],
            Instruction::Load(v) => vec![*v],
            Instruction::Log {
                log_val, log_id, ..
            } => vec![*log_val, *log_id],
            Instruction::MemCopy {
                dst_val,
                src_val,
                byte_len: _,
            } => {
                vec![*dst_val, *src_val]
            }
            Instruction::Nop => vec![],
            Instruction::ReadRegister(_) => vec![],
            Instruction::Ret(v, _) => vec![*v],
            Instruction::Revert(v) => vec![*v],
            Instruction::StateLoadQuadWord { load_val, key } => vec![*load_val, *key],
            Instruction::StateLoadWord(key) => vec![*key],
            Instruction::StateStoreQuadWord { stored_val, key } => vec![*stored_val, *key],
            Instruction::StateStoreWord { stored_val, key } => vec![*stored_val, *key],
            Instruction::Store {
                dst_val,
                stored_val,
            } => {
                vec![*dst_val, *stored_val]
            }
        }
    }

    // Go through each instruction and update use_count.
    for (block, inst) in function.instruction_iter(context) {
        let opds = get_operands(inst.get_instruction(context).unwrap());
        for v in opds {
            match context.values[v.0].value {
                ValueDatum::Instruction(_) => {
                    num_uses
                        .entry(v)
                        .and_modify(|(_block, count)| *count += 1)
                        .or_insert((block, 1));
                }
                ValueDatum::Constant(_) | ValueDatum::Argument(_) => (),
            }
        }
    }

    let mut worklist = function
        .instruction_iter(context)
        .filter(|(_block, inst)| num_uses.get(inst).is_none())
        .collect::<Vec<_>>();

    let mut modified = false;
    while !worklist.is_empty() {
        let (in_block, dead) = worklist.pop().unwrap();
        if !can_eliminate_instruction(context, dead) {
            continue;
        }
        // Process dead's operands.
        let opds = get_operands(dead.get_instruction(context).unwrap());
        for v in opds {
            // Reduce the use count of v. If it reaches 0, add it to the worklist.
            match context.values[v.0].value {
                ValueDatum::Instruction(_) => {
                    let (block, nu) = num_uses.get_mut(&v).unwrap();
                    *nu -= 1;
                    if *nu == 0 {
                        worklist.push((*block, v));
                    }
                }
                ValueDatum::Constant(_) | ValueDatum::Argument(_) => (),
            }
        }

        in_block.remove_instruction(context, dead);
        modified = true;
    }

    Ok(modified)
}

/// Remove entire functions from a module based on whether they are called or not, using a list of
/// root 'entry' functions to perform a search.
///
/// Functions which are `pub` will not be removed and only functions within the passed [`Module`]
/// are considered for removal.
pub fn func_dce(context: &mut Context, module: &Module, entry_fns: &[Function]) -> bool {
    // Recursively find all the functions called by an entry function.
    fn grow_called_function_set(
        context: &Context,
        caller: Function,
        called_set: &mut HashSet<Function>,
    ) {
        if called_set.insert(caller) {
            // We haven't seen caller before.  Iterate for all that it calls.
            for func in caller
                .instruction_iter(context)
                .filter_map(|(_block, ins_value)| {
                    ins_value
                        .get_instruction(context)
                        .and_then(|ins| match ins {
                            Instruction::Call(f, _args) => Some(f),
                            _otherwise => None,
                        })
                })
            {
                grow_called_function_set(context, *func, called_set);
            }
        }
    }

    // Gather our entry functions together into a set.
    let mut called_fns: HashSet<Function> = HashSet::new();
    for entry_fn in entry_fns {
        grow_called_function_set(context, *entry_fn, &mut called_fns);
    }

    // Gather the functions in the module which aren't called.  It's better to collect them
    // separately first so as to avoid any issues with invalidating the function iterator.
    let dead_fns = module
        .function_iter(context)
        .filter(|f| !called_fns.contains(f))
        .collect::<Vec<_>>();

    let modified = !dead_fns.is_empty();
    for dead_fn in dead_fns {
        module.remove_function(context, &dead_fn);
    }

    modified
}
