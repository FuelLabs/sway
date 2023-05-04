use std::collections::hash_map::Entry;

///! Constant value demotion.
///!
///! This pass demotes 'by-value' constant types to 'by-reference` pointer types, based on target
///! specific parameters.
///!
///! Storage for constant values is created on the stack in variables which are initialized with the
///! original values.
use crate::{
    AnalysisResults, Block, Constant, Context, Function, Instruction, IrError, Pass,
    PassMutability, ScopedPass, Value,
};

use rustc_hash::FxHashMap;

pub const CONSTDEMOTION_NAME: &str = "constdemotion";

pub fn create_const_demotion_pass() -> Pass {
    Pass {
        name: CONSTDEMOTION_NAME,
        descr: "By-value constant demotion to by-reference.",
        deps: Vec::new(),
        runner: ScopedPass::FunctionPass(PassMutability::Transform(const_demotion)),
    }
}

pub fn const_demotion(
    context: &mut Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    // Find all candidate constant values and their wrapped constants.
    let mut candidate_values: FxHashMap<Block, Vec<(Value, Constant)>> = FxHashMap::default();

    for (block, inst) in function.instruction_iter(context) {
        let operands = inst.get_instruction(context).unwrap().get_operands();
        for val in operands.iter() {
            if let Some(c) = val.get_constant(context) {
                if super::target_fuel::is_demotable_type(context, &c.ty) {
                    let dem = (*val, c.clone());
                    match candidate_values.entry(block) {
                        Entry::Occupied(mut occ) => {
                            occ.get_mut().push(dem);
                        }
                        Entry::Vacant(vac) => {
                            vac.insert(vec![dem]);
                        }
                    }
                }
            }
        }
    }

    if candidate_values.is_empty() {
        return Ok(false);
    }

    for (block, cands) in candidate_values {
        let mut replace_map: FxHashMap<Value, Value> = FxHashMap::default();
        // The new instructions we're going to insert at the start of this block.
        let mut this_block_new = Vec::new();
        for (c_val, c) in cands {
            // Create a variable for const.
            let var = function.new_unique_local_var(
                context,
                "__const".to_owned(),
                c.ty,
                Some(c.clone()),
                false,
            );
            let var_val = Value::new_instruction(context, Instruction::GetLocal(var));
            let load_val = Value::new_instruction(context, Instruction::Load(var_val));
            replace_map.insert(c_val, load_val);
            this_block_new.push(var_val);
            this_block_new.push(load_val);
        }
        block.replace_values(context, &replace_map);
        block.prepend_instructions(context, this_block_new);
    }

    Ok(true)
}
