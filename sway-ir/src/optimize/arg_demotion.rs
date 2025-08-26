/// Function argument demotion.
///
/// This pass demotes 'by-value' function arg types to 'by-reference` pointer types, based on target
/// specific parameters.
use crate::{
    AnalysisResults, Block, BlockArgument, Context, Function, InstOp, Instruction,
    InstructionInserter, IrError, Module, Pass, PassMutability, ScopedPass, Type, Value,
    ValueDatum,
};

use rustc_hash::FxHashMap;

pub const ARG_DEMOTION_NAME: &str = "arg-demotion";

pub fn create_arg_demotion_pass() -> Pass {
    Pass {
        name: ARG_DEMOTION_NAME,
        descr: "Demotion of by-value function arguments to by-reference",
        deps: Vec::new(),
        runner: ScopedPass::ModulePass(PassMutability::Transform(arg_demotion)),
    }
}

pub fn arg_demotion(
    context: &mut Context,
    _: &AnalysisResults,
    module: Module,
) -> Result<bool, IrError> {
    let mut result = false;
    // This is a module pass because modifying the signature of a function may affect the
    // call sites in other functions, requiring their modification as well.
    for function in module.function_iter(context) {
        result |= fn_arg_demotion(context, function)?;

        // We also need to be sure that block args within this function are demoted.
        for block in function.block_iter(context) {
            result |= demote_block_signature(context, &function, block);
        }
    }

    Ok(result)
}

fn fn_arg_demotion(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // The criteria for now for demotion is whether the arg type is larger than 64-bits or is an
    // aggregate.  This info should be instead determined by a target info analysis pass.

    // Find candidate argument indices.
    let candidate_args = function
        .args_iter(context)
        .enumerate()
        .filter_map(|(idx, (_name, arg_val))| {
            arg_val.get_type(context).and_then(|ty| {
                super::target_fuel::is_demotable_type(context, &ty).then_some((idx, ty))
            })
        })
        .collect::<Vec<(usize, Type)>>();

    if candidate_args.is_empty() {
        return Ok(false);
    }

    // Find all the call sites for this function.
    let call_sites = context
        .module_iter()
        .flat_map(|module| module.function_iter(context))
        .flat_map(|function| function.block_iter(context))
        .flat_map(|block| {
            block
                .instruction_iter(context)
                .filter_map(|instr_val| {
                    if let InstOp::Call(call_to_func, _) = instr_val
                        .get_instruction(context)
                        .expect("`instruction_iter()` must return instruction values.")
                        .op
                    {
                        (call_to_func == function).then_some((block, instr_val))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<(Block, Value)>>();

    // Demote the function signature and the arg uses.
    demote_fn_signature(context, &function, &candidate_args);

    // We need to convert the caller arg value at *every* call site from a by-value to a
    // by-reference.  To do this we create local storage for the value, store it to the variable
    // and pass a pointer to it.
    for (call_block, call_val) in call_sites {
        demote_caller(context, &function, call_block, call_val, &candidate_args);
    }

    Ok(true)
}

fn demote_fn_signature(context: &mut Context, function: &Function, arg_idcs: &[(usize, Type)]) {
    // Change the types of the arg values in place to their pointer counterparts.
    let entry_block = function.get_entry_block(context);
    let old_arg_vals = arg_idcs
        .iter()
        .map(|(arg_idx, arg_ty)| {
            let ptr_ty = Type::new_typed_pointer(context, *arg_ty);

            // Create a new block arg, same as the old one but with a different type.
            let blk_arg_val = entry_block
                .get_arg(context, *arg_idx)
                .expect("Entry block args should be mirror of function args.");
            let ValueDatum::Argument(block_arg) = context.values[blk_arg_val.0].value else {
                panic!("Block argument is not of right Value kind");
            };
            let new_blk_arg_val = Value::new_argument(
                context,
                BlockArgument {
                    ty: ptr_ty,
                    ..block_arg
                },
            );

            // Set both function and block arg to the new one.
            entry_block.set_arg(context, new_blk_arg_val);
            let (_name, fn_arg_val) = &mut context.functions[function.0].arguments[*arg_idx];
            *fn_arg_val = new_blk_arg_val;

            (blk_arg_val, new_blk_arg_val)
        })
        .collect::<Vec<_>>();

    // For each of the old args, which have had their types changed, insert a `load` instruction.
    let mut replace_map = FxHashMap::default();
    let mut new_inserts = Vec::new();
    for (old_arg_val, new_arg_val) in old_arg_vals {
        let load_from_new_arg =
            Value::new_instruction(context, entry_block, InstOp::Load(new_arg_val));
        new_inserts.push(load_from_new_arg);
        replace_map.insert(old_arg_val, load_from_new_arg);
    }

    entry_block.prepend_instructions(context, new_inserts);

    // Replace all uses of the old arg with the loads.
    function.replace_values(context, &replace_map, None);
}

fn demote_caller(
    context: &mut Context,
    function: &Function,
    call_block: Block,
    call_val: Value,
    arg_idcs: &[(usize, Type)],
) {
    // For each argument we update its type by storing the original value to a local variable and
    // passing its pointer.  We return early above if arg_idcs is empty but reassert it here to be
    // sure.
    assert!(!arg_idcs.is_empty());

    // Grab the original args and copy them.
    let Some(Instruction {
        op: InstOp::Call(_, args),
        ..
    }) = call_val.get_instruction(context)
    else {
        unreachable!("`call_val` is definitely a call instruction.");
    };

    // Create a copy of the args to be updated.  And use a new vec of instructions to insert to
    // avoid borrowing the block instructions mutably in the loop.
    let mut args = args.clone();
    let mut new_instrs = Vec::with_capacity(arg_idcs.len() * 2);

    let call_function = call_block.get_function(context);
    for (arg_idx, arg_ty) in arg_idcs {
        // First we make a new local variable.
        let loc_var = call_function.new_unique_local_var(
            context,
            "__tmp_arg".to_owned(),
            *arg_ty,
            None,
            false,
        );
        let get_loc_val = Value::new_instruction(context, call_block, InstOp::GetLocal(loc_var));

        // Before the call we store the original arg value to the new local var.
        let store_val = Value::new_instruction(
            context,
            call_block,
            InstOp::Store {
                dst_val_ptr: get_loc_val,
                stored_val: args[*arg_idx],
            },
        );

        // Use the local var as the new arg.
        args[*arg_idx] = get_loc_val;

        // Insert the new `get_local` and the `store`.
        new_instrs.push(get_loc_val);
        new_instrs.push(store_val);
    }

    // Replace call with the new one with updated args.
    let new_call_val = Value::new_instruction(context, call_block, InstOp::Call(*function, args));
    call_block
        .replace_instruction(context, call_val, new_call_val, false)
        .unwrap();

    // Insert new_instrs before the call.
    let mut inserter = InstructionInserter::new(
        context,
        call_block,
        crate::InsertionPosition::Before(new_call_val),
    );
    inserter.insert_slice(&new_instrs);

    // Replace the old call with the new call.
    call_function.replace_value(context, call_val, new_call_val, None);
}

fn demote_block_signature(context: &mut Context, function: &Function, block: Block) -> bool {
    let candidate_args = block
        .arg_iter(context)
        .enumerate()
        .filter_map(|(idx, arg_val)| {
            arg_val.get_type(context).and_then(|ty| {
                super::target_fuel::is_demotable_type(context, &ty).then_some((idx, *arg_val, ty))
            })
        })
        .collect::<Vec<_>>();

    if candidate_args.is_empty() {
        return false;
    }

    let mut replace_map = FxHashMap::default();
    let mut new_inserts = Vec::new();
    // Update the block signature for each candidate arg.  Create a replacement load for each one.
    for (_arg_idx, arg_val, arg_ty) in &candidate_args {
        let ptr_ty = Type::new_typed_pointer(context, *arg_ty);

        // Create a new block arg, same as the old one but with a different type.
        let ValueDatum::Argument(block_arg) = context.values[arg_val.0].value else {
            panic!("Block argument is not of right Value kind");
        };
        let new_blk_arg_val = Value::new_argument(
            context,
            BlockArgument {
                ty: ptr_ty,
                ..block_arg
            },
        );
        block.set_arg(context, new_blk_arg_val);

        let load_val = Value::new_instruction(context, block, InstOp::Load(new_blk_arg_val));
        new_inserts.push(load_val);
        replace_map.insert(*arg_val, load_val);
    }

    block.prepend_instructions(context, new_inserts);
    // Replace the arg uses with the loads.
    function.replace_values(context, &replace_map, None);

    // Find the predecessors to this block and for each one use a temporary and pass its address to
    // this block. We create a temporary for each block argument and they can be 'shared' between
    // different predecessors since only one at a time can be the actual predecessor.
    let arg_vars = candidate_args
        .into_iter()
        .map(|(idx, arg_val, arg_ty)| {
            let local_var = function.new_unique_local_var(
                context,
                "__tmp_block_arg".to_owned(),
                arg_ty,
                None,
                false,
            );
            (idx, arg_val, local_var)
        })
        .collect::<Vec<(usize, Value, crate::LocalVar)>>();

    let preds = block.pred_iter(context).copied().collect::<Vec<Block>>();
    for pred in preds {
        for (arg_idx, _arg_val, arg_var) in &arg_vars {
            // Get the value which is being passed to the block at this index.
            let arg_val = pred.get_succ_params(context, &block)[*arg_idx];

            // Insert a `get_local` and `store` for each candidate argument and insert them at the
            // end of this block, before the terminator.
            let get_local_val = Value::new_instruction(context, pred, InstOp::GetLocal(*arg_var));
            let store_val = Value::new_instruction(
                context,
                pred,
                InstOp::Store {
                    dst_val_ptr: get_local_val,
                    stored_val: arg_val,
                },
            );

            let mut inserter = InstructionInserter::new(
                context,
                pred,
                crate::InsertionPosition::At(pred.num_instructions(context) - 1),
            );
            inserter.insert_slice(&[get_local_val, store_val]);

            // Replace the use of the old argument with the `get_local` pointer value.
            let term_val = pred
                .get_terminator_mut(context)
                .expect("A predecessor must have a terminator");
            term_val.replace_values(&FxHashMap::from_iter([(arg_val, get_local_val)]));
        }
    }

    true
}
