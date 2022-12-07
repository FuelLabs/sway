//! ## Dead Code Elimination
//!
//! This optimization removes unused definitions. The pass is a combination of
//!   1. A liveness analysis that keeps track of the uses of a definition,
//!   2. At the time of inspecting a definition, if it has no uses, it is removed.
//! This pass does not do CFG transformations. That is handled by simplify_cfg.

use crate::{Block, Context, Function, Instruction, IrError, Module, Value, ValueDatum};

use std::collections::{HashMap, HashSet};

fn can_eliminate_instruction(context: &Context, val: Value) -> bool {
    let inst = val.get_instruction(context).unwrap();
    !inst.is_terminator() && !inst.may_have_side_effect()
}

/// Perform dead code (if any) elimination and return true if function modified.
pub fn dce(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    // Number of uses that an instruction has.
    let mut num_uses: HashMap<Value, (Block, u32)> = HashMap::new();

    // Go through each instruction and update use_count.
    for (block, inst) in function.instruction_iter(context) {
        let opds = inst.get_instruction(context).unwrap().get_operands();
        for v in opds {
            match context.values[v.0].value {
                ValueDatum::Instruction(_) => {
                    num_uses
                        .entry(v)
                        .and_modify(|(_block, count)| *count += 1)
                        .or_insert((block, 1));
                }
                ValueDatum::Constant(_) | ValueDatum::Configurable(_) | ValueDatum::Argument(_) => {
                }
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
        let opds = dead.get_instruction(context).unwrap().get_operands();
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
                ValueDatum::Constant(_) | ValueDatum::Configurable(_) | ValueDatum::Argument(_) => {
                }
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
