//! ## Dead Code Elimination
//!
//! This optimization removes unused definitions. The pass is a combination of
//!   1. A liveness analysis that keeps track of the uses of a definition,
//!   2. At the time of inspecting a definition, if it has no uses, it is removed.
//! This pass does not do CFG transformations. That is handled by simplify_cfg.

use crate::{
    context::Context, error::IrError, function::Function, instruction::Instruction,
    value::ValueDatum, Block, Value,
};

use std::collections::HashMap;

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
            Instruction::Branch(_) => vec![],
            Instruction::Call(_, vs) => vs.clone(),
            Instruction::Cmp(_, lhs, rhs) => vec![*lhs, *rhs],
            Instruction::ConditionalBranch {
                cond_value,
                true_block: _,
                false_block: _,
            } => vec![*cond_value],
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
            Instruction::Nop => vec![],
            Instruction::Phi(ins) => ins.iter().map(|v| v.1).collect(),
            Instruction::ReadRegister(_) => vec![],
            Instruction::Ret(v, _) => vec![*v],
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
        // Don't remove PHIs, just make them empty.
        if matches!(
            &context.values[dead.0].value,
            ValueDatum::Instruction(Instruction::Phi(_))
        ) {
            dead.replace(context, ValueDatum::Instruction(Instruction::Phi(vec![])));
        } else {
            in_block.remove_instruction(context, dead);
        }
        modified = true;
    }

    Ok(modified)
}
