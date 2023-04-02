///! Return value demotion.
///!
///! This pass demotes 'by-value' function return types to 'by-reference` pointer types, based on
///! target specific parameters.
///!
///! An extra argument pointer is added to the function and this pointer is also returned.  The
///! return value is mem_copied to the new argument instead of being returned by value.
use crate::{
    AnalysisResults, BlockArgument, Context, Function, Instruction, IrError, Pass, PassMutability,
    ScopedPass, Type, Value,
};

pub const RETDEMOTION_NAME: &str = "retdemotion";

pub fn create_ret_demotion_pass() -> Pass {
    Pass {
        name: RETDEMOTION_NAME,
        descr: "By-value function return value demotion to by-reference.",
        deps: Vec::new(),
        runner: ScopedPass::FunctionPass(PassMutability::Transform(ret_val_demotion)),
    }
}

pub fn ret_val_demotion(
    context: &mut Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    // Reject non-candidate.
    let ret_type = function.get_return_type(context);
    if !super::target_fuel::is_demotable_type(context, &ret_type) {
        // Return type fits in a register.
        return Ok(false);
    }

    // Change the function signature.  It now returns a pointer.
    let ptr_ret_type = Type::new_ptr(context, ret_type);
    function.set_return_type(context, ptr_ret_type);

    // The storage for the return value must be determined.  For entry-point functions it's a new
    // local and otherwise it's an extra argument.
    let entry_block = function.get_entry_block(context);
    let ptr_arg_val = if function.is_entry(context) {
        let ret_var =
            function.new_unique_local_var(context, "__ret_value".to_owned(), ret_type, None);

        // Insert the return value pointer at the start of the entry block.
        let get_ret_var = Value::new_instruction(context, Instruction::GetLocal(ret_var));
        context.blocks[entry_block.0]
            .instructions
            .insert(0, get_ret_var);
        get_ret_var
    } else {
        let ptr_arg_val = Value::new_argument(
            context,
            BlockArgument {
                block: entry_block,
                idx: function.num_args(context),
                ty: ptr_ret_type,
            },
        );
        function.add_arg(context, "__ret_value", ptr_arg_val);
        entry_block.add_arg(context, ptr_arg_val);
        ptr_arg_val
    };

    // Gather the blocks which are returning.
    let ret_blocks = function
        .block_iter(context)
        .filter_map(|block| {
            block.get_terminator(context).and_then(|term| {
                if let Instruction::Ret(ret_val, _ty) = term {
                    Some((block, *ret_val))
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    // Update each `ret` to store the return value to the 'out' arg and then return the pointer.
    for (ret_block, ret_val) in ret_blocks {
        // This is a special case where we're replacing the terminator.  We can just pop it off the
        // end of the block and add new instructions.
        let block_instrs = &mut context.blocks[ret_block.0].instructions;
        let orig_ret_val = block_instrs.last().copied();
        block_instrs.pop();
        let md_idx = orig_ret_val.and_then(|val| val.get_metadata(context));

        ret_block
            .ins(context)
            .store(ptr_arg_val, ret_val)
            .add_metadatum(context, md_idx);
        ret_block
            .ins(context)
            .ret(ptr_arg_val, ptr_ret_type)
            .add_metadatum(context, md_idx);
    }

    // If the function isn't an entry point we need to update all the callers to pass the extra
    // argument.
    if !function.is_entry(context) {
        update_callers(context, function, ret_type);
    }

    Ok(true)
}

fn update_callers(context: &mut Context, function: Function, ret_type: Type) {
    // Now update all the callers to pass the return value argument. Find all the call sites for
    // this function.
    let call_sites = context
        .module_iter()
        .flat_map(|module| module.function_iter(context))
        .flat_map(|ref call_from_func| {
            call_from_func
                .block_iter(context)
                .flat_map(|ref block| {
                    block
                        .instruction_iter(context)
                        .filter_map(|instr_val| {
                            if let Instruction::Call(call_to_func, _) = instr_val
                                .get_instruction(context)
                                .expect("`instruction_iter()` must return instruction values.")
                            {
                                (*call_to_func == function).then_some((
                                    *call_from_func,
                                    *block,
                                    instr_val,
                                ))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // Create a local var to receive the return value for each call site.  Replace the `call`
    // instruction with a `get_local`, an updated `call` and a `load`.
    for (calling_func, calling_block, call_val) in call_sites {
        // First make a new local variable.
        let loc_var =
            calling_func.new_unique_local_var(context, "__ret_val".to_owned(), ret_type, None);
        let get_loc_val = Value::new_instruction(context, Instruction::GetLocal(loc_var));

        // Next we need to copy the original `call` but add the extra arg.
        let Some(Instruction::Call(_, args)) = call_val.get_instruction(context) else {
            unreachable!("`call_val` is definitely a call instruction.");
        };
        let mut new_args = args.clone();
        new_args.push(get_loc_val);
        let new_call_val = Value::new_instruction(context, Instruction::Call(function, new_args));

        // And finally load the value from the new local var.
        let load_val = Value::new_instruction(context, Instruction::Load(new_call_val));

        // We don't have an actual instruction _inserter_ yet, just an appender, so we need to do
        // this manually.
        let block_instrs = &mut context.blocks[calling_block.0].instructions;
        let call_inst_idx = block_instrs
            .iter()
            .position(|&instr_val| instr_val == call_val)
            .unwrap();

        // Replace the call with the new `get_local` then insert the `call` and the `load` after
        // it.
        block_instrs[call_inst_idx] = get_loc_val;
        block_instrs.insert(call_inst_idx + 1, new_call_val);
        block_instrs.insert(call_inst_idx + 2, load_val);

        // Replace the old call with the new load.
        calling_func.replace_value(context, call_val, load_val, None);
    }
}
