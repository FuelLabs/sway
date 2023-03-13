///! Miscellaneous value demotion.
///!
///! This pass demotes miscellaneous 'by-value' types to 'by-reference' pointer types, based on
///! target specific parameters.
///!
///! Current special cases are:
///! - log arguments: These can be any type and should be demoted to pointers if possible.
///! - Fuel ASM block arguments: These are assumed to be pointers for 'by-reference' values.
///! - Fuel ASM block return values: These are also assumed to be pointers for 'by-reference'
///!   values.
use crate::{
    asm::AsmArg, AnalysisResults, Context, FuelVmInstruction, Function, Instruction, IrError, Pass,
    PassMutability, ScopedPass, Type, TypeContent, Value,
};

use rustc_hash::FxHashMap;

pub const MISCDEMOTION_NAME: &str = "miscdemotion";

pub fn create_misc_demotion_pass() -> Pass {
    Pass {
        name: MISCDEMOTION_NAME,
        descr: "By-value miscellaneous demotion to by-reference.",
        deps: Vec::new(),
        runner: ScopedPass::FunctionPass(PassMutability::Transform(misc_demotion)),
    }
}

pub fn misc_demotion(
    context: &mut Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let log_res = log_demotion(context, function)?;
    let asm_arg_res = asm_block_arg_demotion(context, function)?;
    let asm_ret_res = asm_block_ret_demotion(context, function)?;

    Ok(log_res || asm_arg_res || asm_ret_res)
}

fn log_demotion(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find all log instructions.
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            instr_val.get_instruction(context).and_then(|instr| {
                // Is the instruction a Log?
                if let Instruction::FuelVm(FuelVmInstruction::Log {
                    log_val,
                    log_ty,
                    log_id,
                }) = instr
                {
                    is_demote_type(context, log_ty)
                        .then_some((block, instr_val, *log_val, *log_ty, *log_id))
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    // Take the logged value, store it in a temporary local, and replace it with its pointer in the
    // log instruction.
    for (block, log_instr_val, logged_val, logged_ty, log_id_val) in candidates {
        // Create a variable for the arg, a get_local for it and a store.
        let loc_var =
            function.new_unique_local_var(context, "__log_arg".to_owned(), logged_ty, None);
        let get_loc_val = Value::new_instruction(context, Instruction::GetLocal(loc_var));
        let store_val = Value::new_instruction(
            context,
            Instruction::Store {
                dst_val_ptr: get_loc_val,
                stored_val: logged_val,
            },
        );

        // We need to replace the log instruction because we're changing the type to a pointer.
        let ptr_ty = Type::new_ptr(context, logged_ty);
        let new_log_instr_val = Value::new_instruction(
            context,
            Instruction::FuelVm(FuelVmInstruction::Log {
                log_val: get_loc_val,
                log_ty: ptr_ty,
                log_id: log_id_val,
            }),
        );

        // We don't have an actual instruction _inserter_ yet, just an appender, so we need to find the
        // log instruction index and insert instructions manually.
        let block_instrs = &mut context.blocks[block.0].instructions;
        let log_inst_idx = block_instrs
            .iter()
            .position(|&instr_val| instr_val == log_instr_val)
            .unwrap();
        block_instrs[log_inst_idx] = new_log_instr_val;

        // Put these two _before_ it.
        block_instrs.insert(log_inst_idx, get_loc_val);
        block_instrs.insert(log_inst_idx + 1, store_val);

        // NOTE: We don't need to replace the uses of the old log instruction as it doesn't return a
        // value.  (It's a 'statement' rather than an 'expression'.)
    }

    Ok(true)
}

fn asm_block_arg_demotion(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Gather the ASM blocks with reference type args.
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            instr_val.get_instruction(context).and_then(|instr| {
                // Is the instruction an ASM block?
                if let Instruction::AsmBlock(_asm_block, args) = instr {
                    let ref_args = args
                        .iter()
                        .filter_map(
                            |AsmArg {
                                 name: _,
                                 initializer,
                             }| {
                                initializer.and_then(|init_val| {
                                    init_val.get_type(context).and_then(|ty| {
                                        is_demote_type(context, &ty).then_some((init_val, ty))
                                    })
                                })
                            },
                        )
                        .collect::<Vec<_>>();

                    (!ref_args.is_empty()).then_some((block, instr_val, ref_args))
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    for (block, asm_block_instr_val, ref_args) in candidates {
        let temporaries = ref_args
            .iter()
            .map(|(ref_arg_val, ref_arg_ty)| {
                // Create temporaries for each of the by-reference args.
                let loc_var = function.new_unique_local_var(
                    context,
                    "__asm_arg".to_owned(),
                    *ref_arg_ty,
                    None,
                );

                // Create `get_local`s and `store`s for each one.
                let get_loc_val = Value::new_instruction(context, Instruction::GetLocal(loc_var));
                let store_val = Value::new_instruction(
                    context,
                    Instruction::Store {
                        dst_val_ptr: get_loc_val,
                        stored_val: *ref_arg_val,
                    },
                );

                (*ref_arg_val, get_loc_val, store_val)
            })
            .collect::<Vec<(Value, Value, Value)>>();

        // Insert the temporaries into the block. Again, we don't have an actual instruction
        // _inserter_ yet.
        let block_instrs = &mut context.blocks[block.0].instructions;
        let asm_inst_idx = block_instrs
            .iter()
            .position(|&instr_val| instr_val == asm_block_instr_val)
            .unwrap();
        for (idx, (_ref_arg_val, get_loc_val, store_val)) in temporaries.iter().enumerate() {
            block_instrs.insert(asm_inst_idx + idx * 2, *get_loc_val);
            block_instrs.insert(asm_inst_idx + idx * 2 + 1, *store_val);
        }

        // Replace the args with the `get_local`s in the ASM block.
        asm_block_instr_val.replace_instruction_values(
            context,
            &FxHashMap::from_iter(
                temporaries
                    .into_iter()
                    .map(|(ref_arg_val, get_loc_val, _store_val)| (ref_arg_val, get_loc_val)),
            ),
        );
    }

    Ok(true)
}

fn asm_block_ret_demotion(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Gather the ASM blocks which return a reference type.
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            instr_val.get_instruction(context).and_then(|instr| {
                // Is the instruction an ASM block?
                if let Instruction::AsmBlock(asm_block, _args) = instr {
                    let ret_ty = asm_block.get_type(context);
                    is_demote_type(context, &ret_ty)
                        .then_some((block, instr_val, *asm_block, ret_ty))
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    let mut replace_map = FxHashMap::default();
    for (block, asm_block_instr_val, asm_block, ret_ty) in candidates {
        // Change the ASM block return type to be a pointer.
        let ret_ptr_ty = Type::new_ptr(context, ret_ty);
        asm_block.set_type(context, ret_ptr_ty);

        // Insert a load after the block.  Still no instruction inserter...
        let load_val = Value::new_instruction(context, Instruction::Load(asm_block_instr_val));
        let block_instrs = &mut context.blocks[block.0].instructions;
        let asm_inst_idx = block_instrs
            .iter()
            .position(|&instr_val| instr_val == asm_block_instr_val)
            .unwrap();
        block_instrs.insert(asm_inst_idx + 1, load_val);

        // Replace uses of the ASM block with the new load.
        replace_map.insert(asm_block_instr_val, load_val);
    }
    function.replace_values(context, &replace_map, None);

    Ok(true)
}

fn is_demote_type(context: &Context, ty: &Type) -> bool {
    match ty.get_content(context) {
        TypeContent::Unit | TypeContent::Bool | TypeContent::Pointer(_) => false,
        TypeContent::Uint(bits) => *bits > 64,
        _ => true,
    }
}
