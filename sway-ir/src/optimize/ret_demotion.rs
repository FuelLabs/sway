/// Return value demotion.
///
/// This pass demotes 'by-value' function return types to 'by-reference` pointer types, based on
/// target specific parameters.
///
/// An extra argument pointer is added to the function.
/// The return value is mem_copied to the new argument instead of being returned by value.
use crate::{
    AnalysisResults, BlockArgument, ConstantContent, Context, Function, InstOp, Instruction,
    InstructionInserter, IrError, Module, Pass, PassMutability, ScopedPass, Type, Value,
};

pub const RET_DEMOTION_NAME: &str = "ret-demotion";

pub fn create_ret_demotion_pass() -> Pass {
    Pass {
        name: RET_DEMOTION_NAME,
        descr: "Demotion of by-value function return values to by-reference",
        deps: Vec::new(),
        runner: ScopedPass::ModulePass(PassMutability::Transform(ret_val_demotion)),
    }
}

pub fn ret_val_demotion(
    context: &mut Context,
    _analyses: &AnalysisResults,
    module: Module,
) -> Result<bool, IrError> {
    // This is a module pass because we need to update all the callers of a function if we change
    // its signature.
    let mut changed = false;
    for function in module.function_iter(context) {
        // Reject non-candidate.
        let ret_type = function.get_return_type(context);
        if !super::target_fuel::is_demotable_type(context, &ret_type) {
            // Return type fits in a register.
            continue;
        }

        changed = true;

        // Change the function signature.
        let ptr_ret_type = Type::new_typed_pointer(context, ret_type);
        let unit_ty = Type::get_unit(context);

        // The storage for the return value must be determined.  For entry-point functions it's a new
        // local and otherwise it's an extra argument.
        let entry_block = function.get_entry_block(context);
        let ptr_arg_val = if function.is_entry(context) {
            // Entry functions return a pointer to the original return type.
            function.set_return_type(context, ptr_ret_type);

            // Create a local variable to hold the return value.
            let ret_var = function.new_unique_local_var(
                context,
                "__ret_value".to_owned(),
                ret_type,
                None,
                false,
            );

            // Insert the return value pointer at the start of the entry block.
            let get_ret_var =
                Value::new_instruction(context, entry_block, InstOp::GetLocal(ret_var));
            entry_block.prepend_instructions(context, vec![get_ret_var]);
            get_ret_var
        } else {
            // non-entry functions now return unit.
            function.set_return_type(context, unit_ty);

            let ptr_arg_val = Value::new_argument(
                context,
                BlockArgument {
                    block: entry_block,
                    idx: function.num_args(context),
                    ty: ptr_ret_type,
                    is_immutable: false,
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
                    if let InstOp::Ret(ret_val, _ty) = term.op {
                        Some((block, ret_val))
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
            let last_instr_pos = ret_block.num_instructions(context) - 1;
            let orig_ret_val = ret_block.get_instruction_at(context, last_instr_pos);
            ret_block.remove_instruction_at(context, last_instr_pos);
            let md_idx = orig_ret_val.and_then(|val| val.get_metadata(context));

            ret_block
                .append(context)
                .store(ptr_arg_val, ret_val)
                .add_metadatum(context, md_idx);

            if !function.is_entry(context) {
                let unit_ret = ConstantContent::get_unit(context);
                ret_block
                    .append(context)
                    .ret(unit_ret, unit_ty)
                    .add_metadatum(context, md_idx);
            } else {
                // Entry functions still return the pointer to the return value.
                ret_block
                    .append(context)
                    .ret(ptr_arg_val, ptr_ret_type)
                    .add_metadatum(context, md_idx);
            }
        }

        // If the function isn't an entry point we need to update all the callers to pass the extra
        // argument.
        if !function.is_entry(context) {
            update_callers(context, function, ret_type);
        }
    }

    Ok(changed)
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
                            if let Instruction {
                                op: InstOp::Call(call_to_func, _),
                                ..
                            } = instr_val
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
        let loc_var = calling_func.new_unique_local_var(
            context,
            "__ret_val".to_owned(),
            ret_type,
            None,
            false,
        );
        let get_loc_val = Value::new_instruction(context, calling_block, InstOp::GetLocal(loc_var));

        // Next we need to copy the original `call` but add the extra arg.
        let Some(Instruction {
            op: InstOp::Call(_, args),
            ..
        }) = call_val.get_instruction(context)
        else {
            unreachable!("`call_val` is definitely a call instruction.");
        };
        let mut new_args = args.clone();
        new_args.push(get_loc_val);
        let new_call_val =
            Value::new_instruction(context, calling_block, InstOp::Call(function, new_args));

        // And finally load the value from the new local var.
        let load_val = Value::new_instruction(context, calling_block, InstOp::Load(get_loc_val));

        calling_block
            .replace_instruction(context, call_val, get_loc_val, false)
            .unwrap();
        let mut inserter = InstructionInserter::new(
            context,
            calling_block,
            crate::InsertionPosition::After(get_loc_val),
        );
        inserter.insert_slice(&[new_call_val, load_val]);

        // Replace the old call with the new load.
        calling_func.replace_value(context, call_val, load_val, None);
    }
}
