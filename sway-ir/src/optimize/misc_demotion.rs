use std::ops::Not;

/// Miscellaneous value demotion.
///
/// This pass demotes miscellaneous 'by-value' types to 'by-reference' pointer types, based on
/// target specific parameters.
///
/// Current special cases are:
/// - log arguments: These can be any type and should be demoted to pointers if possible.
/// - Fuel ASM block arguments: These are assumed to be pointers for 'by-reference' values.
/// - Fuel ASM block return values: These are also assumed to be pointers for 'by-reference'
///   values.
/// - Fuel Wide binary operators: Demote binary operands bigger than 64 bits.
use crate::{
    asm::AsmArg, AnalysisResults, BinaryOpKind, Constant, ConstantContent, Context,
    FuelVmInstruction, Function, InstOp, InstructionInserter, IrError, Pass, PassMutability,
    Predicate, ScopedPass, Type, UnaryOpKind, Value,
};

use rustc_hash::FxHashMap;

pub const MISC_DEMOTION_NAME: &str = "misc-demotion";

pub fn create_misc_demotion_pass() -> Pass {
    Pass {
        name: MISC_DEMOTION_NAME,
        descr: "Miscellaneous by-value demotions to by-reference",
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
    // let addrof_res = ptr_to_int_demotion(context, function)?;

    let wide_binary_op_res = wide_binary_op_demotion(context, function)?;
    let wide_shifts_op_res = wide_shift_op_demotion(context, function)?;
    let wide_cmp_res = wide_cmp_demotion(context, function)?;
    let wide_unary_op_res = wide_unary_op_demotion(context, function)?;

    Ok(log_res
        || asm_arg_res
        || asm_ret_res
        || wide_unary_op_res
        || wide_binary_op_res
        || wide_shifts_op_res
        || wide_cmp_res)
}

fn log_demotion(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find all log instructions.
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            instr_val.get_instruction(context).and_then(|instr| {
                // Is the instruction a Log?
                if let InstOp::FuelVm(FuelVmInstruction::Log {
                    log_val,
                    log_ty,
                    log_id,
                }) = instr.op
                {
                    super::target_fuel::is_demotable_type(context, &log_ty)
                        .then_some((block, instr_val, log_val, log_ty, log_id))
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
            function.new_unique_local_var(context, "__log_arg".to_owned(), logged_ty, None, false);
        let get_loc_val = Value::new_instruction(context, block, InstOp::GetLocal(loc_var));
        let store_val = Value::new_instruction(
            context,
            block,
            InstOp::Store {
                dst_val_ptr: get_loc_val,
                stored_val: logged_val,
            },
        );

        // We need to replace the log instruction because we're changing the type to a pointer.
        let ptr_ty = Type::new_ptr(context, logged_ty);
        let new_log_instr_val = Value::new_instruction(
            context,
            block,
            InstOp::FuelVm(FuelVmInstruction::Log {
                log_val: get_loc_val,
                log_ty: ptr_ty,
                log_id: log_id_val,
            }),
        );

        // NOTE: We don't need to replace the uses of the old log instruction as it doesn't return a
        // value.  (It's a 'statement' rather than an 'expression'.)
        block
            .replace_instruction(context, log_instr_val, new_log_instr_val, false)
            .unwrap();

        // Put these two _before_ it.
        let mut inserter = InstructionInserter::new(
            context,
            block,
            crate::InsertionPosition::Before(new_log_instr_val),
        );
        inserter.insert_slice(&[get_loc_val, store_val]);
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
                if let InstOp::AsmBlock(_asm_block, args) = &instr.op {
                    let ref_args = args
                        .iter()
                        .filter_map(
                            |AsmArg {
                                 name: _,
                                 initializer,
                             }| {
                                initializer.and_then(|init_val| {
                                    init_val.get_type(context).and_then(|ty| {
                                        super::target_fuel::is_demotable_type(context, &ty)
                                            .then_some((init_val, ty))
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
        let mut replace_map = FxHashMap::default();
        let mut temporaries = Vec::new();

        for (ref_arg_val, ref_arg_ty) in ref_args {
            // Create temporaries for each of the by-reference args.
            let loc_var = function.new_unique_local_var(
                context,
                "__asm_arg".to_owned(),
                ref_arg_ty,
                None,
                false,
            );

            // Create `get_local`s and `store`s for each one.
            let get_loc_val = Value::new_instruction(context, block, InstOp::GetLocal(loc_var));
            let store_val = Value::new_instruction(
                context,
                block,
                InstOp::Store {
                    dst_val_ptr: get_loc_val,
                    stored_val: ref_arg_val,
                },
            );

            replace_map.insert(ref_arg_val, get_loc_val);
            temporaries.push(get_loc_val);
            temporaries.push(store_val);
        }

        // Insert the temporaries into the block.
        let mut inserter = InstructionInserter::new(
            context,
            block,
            crate::InsertionPosition::Before(asm_block_instr_val),
        );
        inserter.insert_slice(&temporaries);

        // Replace the args with the `get_local`s in the ASM block.
        asm_block_instr_val.replace_instruction_values(context, &replace_map);
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
                if let InstOp::AsmBlock(asm_block, args) = &instr.op {
                    let ret_ty = asm_block.return_type;
                    super::target_fuel::is_demotable_type(context, &ret_ty).then_some((
                        block,
                        instr_val,
                        asm_block.clone(),
                        args.clone(),
                        ret_ty,
                    ))
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
    for (block, asm_block_instr_val, mut asm_block, asm_args, ret_ty) in candidates {
        // Change the ASM block return type to be a pointer.
        let ret_ptr_ty = Type::new_ptr(context, ret_ty);
        asm_block.return_type = ret_ptr_ty;
        let new_asm_block =
            Value::new_instruction(context, block, InstOp::AsmBlock(asm_block, asm_args));

        // Insert a load after the block.
        let load_val = Value::new_instruction(context, block, InstOp::Load(new_asm_block));
        block
            .replace_instruction(context, asm_block_instr_val, new_asm_block, false)
            .unwrap();
        let mut inserter = InstructionInserter::new(
            context,
            block,
            crate::InsertionPosition::After(new_asm_block),
        );
        inserter.insert(load_val);

        // Replace uses of the old ASM block with the new load.
        replace_map.insert(asm_block_instr_val, load_val);
    }
    function.replace_values(context, &replace_map, None);

    Ok(true)
}

fn ptr_to_int_demotion(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find all ptr_to_int instructions, which are generated by the __addr_of() intrinsic.
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            instr_val.get_instruction(context).and_then(|instr| {
                // Is the instruction a PtrToInt?
                if let InstOp::PtrToInt(ptr_val, _int_ty) = instr.op {
                    ptr_val.get_type(context).and_then(|ptr_ty| {
                        super::target_fuel::is_demotable_type(context, &ptr_ty)
                            .then_some((block, instr_val, ptr_val, ptr_ty))
                    })
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    for (block, ptr_to_int_instr_val, ptr_val, ptr_ty) in candidates {
        // If the ptr_val is a load from a memory location, we can just refer to that.
        if let Some(instr) = ptr_val.get_instruction(context) {
            if let Some(loaded_val) = match instr.op {
                InstOp::Load(loaded_val) => Some(loaded_val),
                _ => None,
            } {
                ptr_to_int_instr_val.replace_instruction_value(context, ptr_val, loaded_val);
                continue;
            }
        }

        // Take the ptr_to_int value, store it in a temporary local, and replace it with its pointer in
        // the ptr_to_int instruction.

        // Create a variable for the arg, a get_local for it and a store.
        let loc_var = function.new_unique_local_var(
            context,
            "__ptr_to_int_arg".to_owned(),
            ptr_ty,
            None,
            false,
        );
        let get_loc_val = Value::new_instruction(context, block, InstOp::GetLocal(loc_var));
        let store_val = Value::new_instruction(
            context,
            block,
            InstOp::Store {
                dst_val_ptr: get_loc_val,
                stored_val: ptr_val,
            },
        );

        // Put these two _before_ ptr_to_int_instr_val.
        let mut inserter = InstructionInserter::new(
            context,
            block,
            crate::InsertionPosition::Before(ptr_to_int_instr_val),
        );
        inserter.insert_slice(&[get_loc_val, store_val]);

        // Replace the argument to ptr_to_int.
        ptr_to_int_instr_val.replace_instruction_value(context, ptr_val, get_loc_val);
    }

    Ok(true)
}

/// Find all binary operations on types bigger than 64 bits
/// and demote them to fuel specific `wide binary ops`, that
/// work only on pointers
fn wide_binary_op_demotion(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find all intrinsics on wide operators
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            use BinaryOpKind as B;
            let InstOp::BinaryOp {
                op: B::Add | B::Sub | B::Mul | B::Div | B::Mod | B::And | B::Or | B::Xor,
                arg1,
                arg2,
            } = instr_val.get_instruction(context)?.op
            else {
                return None;
            };

            let arg1_type = arg1.get_type(context);
            let arg2_type = arg2.get_type(context);

            match (arg1_type, arg2_type) {
                (Some(arg1_type), Some(arg2_type))
                    if arg1_type.is_uint_of(context, 256) && arg2_type.is_uint_of(context, 256) =>
                {
                    Some((block, instr_val))
                }
                (Some(arg1_type), Some(arg2_type))
                    if arg1_type.is_b256(context) && arg2_type.is_b256(context) =>
                {
                    Some((block, instr_val))
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    // Now create a local for the result
    // get ptr to each arg
    // and store the result after
    for (block, binary_op_instr_val) in candidates {
        let InstOp::BinaryOp { op, arg1, arg2 } = binary_op_instr_val
            .get_instruction(context)
            .cloned()
            .unwrap()
            .op
        else {
            continue;
        };

        let binary_op_metadata = binary_op_instr_val.get_metadata(context);

        let arg1_ty = arg1.get_type(context).unwrap();
        let arg1_metadata = arg1.get_metadata(context);
        let arg2_ty = arg2.get_type(context).unwrap();
        let arg2_metadata = arg2.get_metadata(context);

        let operand_ty = arg1.get_type(context).unwrap();

        let result_local = function.new_unique_local_var(
            context,
            "__wide_result".to_owned(),
            operand_ty,
            None,
            true,
        );
        let get_result_local =
            Value::new_instruction(context, block, InstOp::GetLocal(result_local))
                .add_metadatum(context, binary_op_metadata);
        let load_result_local =
            Value::new_instruction(context, block, InstOp::Load(get_result_local))
                .add_metadatum(context, binary_op_metadata);

        // If arg1 is not a pointer, store it to a local
        let lhs_store = if !arg1_ty.is_ptr(context) {
            let lhs_local = function.new_unique_local_var(
                context,
                "__wide_lhs".to_owned(),
                operand_ty,
                None,
                false,
            );
            let get_lhs_local = Value::new_instruction(context, block, InstOp::GetLocal(lhs_local))
                .add_metadatum(context, arg1_metadata);
            let store_lhs_local = Value::new_instruction(
                context,
                block,
                InstOp::Store {
                    dst_val_ptr: get_lhs_local,
                    stored_val: arg1,
                },
            )
            .add_metadatum(context, arg1_metadata);
            Some((get_lhs_local, store_lhs_local))
        } else {
            None
        };

        let (arg1_needs_insert, get_arg1) = if let Some((lhs_local, _)) = &lhs_store {
            (false, *lhs_local)
        } else {
            (true, arg1)
        };

        // If arg2 is not a pointer, store it to a local
        let rhs_store = if !arg2_ty.is_ptr(context) {
            let rhs_local = function.new_unique_local_var(
                context,
                "__wide_rhs".to_owned(),
                operand_ty,
                None,
                false,
            );
            let get_rhs_local = Value::new_instruction(context, block, InstOp::GetLocal(rhs_local))
                .add_metadatum(context, arg2_metadata);
            let store_lhs_local = Value::new_instruction(
                context,
                block,
                InstOp::Store {
                    dst_val_ptr: get_rhs_local,
                    stored_val: arg2,
                },
            )
            .add_metadatum(context, arg2_metadata);
            Some((get_rhs_local, store_lhs_local))
        } else {
            None
        };

        let (arg2_needs_insert, get_arg2) = if let Some((rhs_local, _)) = &rhs_store {
            (false, *rhs_local)
        } else {
            (true, arg2)
        };

        // For MOD we need a local zero as RHS of the add operation
        let (wide_op, get_local_zero) = match op {
            BinaryOpKind::Mod => {
                let initializer = ConstantContent::new_uint(context, 256, 0);
                let initializer = Constant::unique(context, initializer);
                let local_zero = function.new_unique_local_var(
                    context,
                    "__wide_zero".to_owned(),
                    operand_ty,
                    Some(initializer),
                    true,
                );
                let get_local_zero =
                    Value::new_instruction(context, block, InstOp::GetLocal(local_zero))
                        .add_metadatum(context, binary_op_metadata);

                (
                    Value::new_instruction(
                        context,
                        block,
                        InstOp::FuelVm(FuelVmInstruction::WideModularOp {
                            op,
                            result: get_result_local,
                            arg1: get_arg1,
                            arg2: get_local_zero,
                            arg3: get_arg2,
                        }),
                    )
                    .add_metadatum(context, binary_op_metadata),
                    Some(get_local_zero),
                )
            }
            _ => (
                Value::new_instruction(
                    context,
                    block,
                    InstOp::FuelVm(FuelVmInstruction::WideBinaryOp {
                        op,
                        arg1: get_arg1,
                        arg2: get_arg2,
                        result: get_result_local,
                    }),
                )
                .add_metadatum(context, binary_op_metadata),
                None,
            ),
        };

        // Assert all operands are pointers
        assert!(get_arg1.get_type(context).unwrap().is_ptr(context));
        assert!(get_arg2.get_type(context).unwrap().is_ptr(context));
        assert!(get_result_local.get_type(context).unwrap().is_ptr(context));
        if let Some(get_local_zero) = &get_local_zero {
            assert!(get_local_zero.get_type(context).unwrap().is_ptr(context));
        }

        block
            .replace_instruction(context, binary_op_instr_val, load_result_local, true)
            .unwrap();

        let mut additional_instrs = Vec::new();

        // lhs
        if let Some((get_lhs_local, store_lhs_local)) = lhs_store {
            additional_instrs.push(get_lhs_local);
            additional_instrs.push(store_lhs_local);
        }
        // Only for MOD
        if let Some(get_local_zero) = get_local_zero {
            additional_instrs.push(get_local_zero);
        }

        //rhs
        if let Some((get_rhs_local, store_rhs_local)) = rhs_store {
            additional_instrs.push(get_rhs_local);
            additional_instrs.push(store_rhs_local);
        }
        if arg1_needs_insert {
            additional_instrs.push(get_arg1);
        }

        if arg2_needs_insert {
            additional_instrs.push(get_arg2);
        }

        additional_instrs.push(get_result_local);
        additional_instrs.push(wide_op);

        let mut inserter = InstructionInserter::new(
            context,
            block,
            crate::InsertionPosition::Before(load_result_local),
        );
        inserter.insert_slice(&additional_instrs);
    }

    Ok(true)
}

/// Find all cmp operations on types bigger than 64 bits
/// and demote them to fuel specific `wide cmp ops`, that
/// work only on pointers
fn wide_cmp_demotion(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find all cmp on wide operators
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            let InstOp::Cmp(
                Predicate::Equal | Predicate::LessThan | Predicate::GreaterThan,
                arg1,
                arg2,
            ) = instr_val.get_instruction(context)?.op
            else {
                return None;
            };

            let arg1_type = arg1.get_type(context);
            let arg2_type = arg2.get_type(context);

            match (arg1_type, arg2_type) {
                (Some(arg1_type), Some(arg2_type))
                    if arg1_type.is_uint_of(context, 256) && arg2_type.is_uint_of(context, 256) =>
                {
                    Some((block, instr_val))
                }
                (Some(arg1_type), Some(arg2_type))
                    if arg1_type.is_b256(context) && arg2_type.is_b256(context) =>
                {
                    Some((block, instr_val))
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    // Get ptr to each arg
    for (block, cmp_instr_val) in candidates {
        let InstOp::Cmp(op, arg1, arg2) =
            cmp_instr_val.get_instruction(context).cloned().unwrap().op
        else {
            continue;
        };

        let cmp_op_metadata = cmp_instr_val.get_metadata(context);

        let arg1_ty = arg1.get_type(context).unwrap();
        let arg1_metadata = arg1.get_metadata(context);
        let arg2_ty = arg2.get_type(context).unwrap();
        let arg2_metadata = arg2.get_metadata(context);

        // If arg1 is not a pointer, store it to a local
        let lhs_store = arg1_ty.is_ptr(context).not().then(|| {
            let lhs_local = function.new_unique_local_var(
                context,
                "__wide_lhs".to_owned(),
                arg1_ty,
                None,
                false,
            );
            let get_lhs_local = Value::new_instruction(context, block, InstOp::GetLocal(lhs_local))
                .add_metadatum(context, arg1_metadata);
            let store_lhs_local = Value::new_instruction(
                context,
                block,
                InstOp::Store {
                    dst_val_ptr: get_lhs_local,
                    stored_val: arg1,
                },
            )
            .add_metadatum(context, arg1_metadata);
            (get_lhs_local, store_lhs_local)
        });

        let (arg1_needs_insert, get_arg1) = if let Some((lhs_local, _)) = &lhs_store {
            (false, *lhs_local)
        } else {
            (true, arg1)
        };

        // If arg2 is not a pointer, store it to a local
        let rhs_store = arg2_ty.is_ptr(context).not().then(|| {
            let rhs_local = function.new_unique_local_var(
                context,
                "__wide_rhs".to_owned(),
                arg1_ty,
                None,
                false,
            );
            let get_rhs_local = Value::new_instruction(context, block, InstOp::GetLocal(rhs_local))
                .add_metadatum(context, arg2_metadata);
            let store_lhs_local = Value::new_instruction(
                context,
                block,
                InstOp::Store {
                    dst_val_ptr: get_rhs_local,
                    stored_val: arg2,
                },
            )
            .add_metadatum(context, arg2_metadata);
            (get_rhs_local, store_lhs_local)
        });

        let (arg2_needs_insert, get_arg2) = if let Some((rhs_local, _)) = &rhs_store {
            (false, *rhs_local)
        } else {
            (true, arg2)
        };

        // Assert all operands are pointers
        assert!(get_arg1.get_type(context).unwrap().is_ptr(context));
        assert!(get_arg2.get_type(context).unwrap().is_ptr(context));

        let wide_op = Value::new_instruction(
            context,
            block,
            InstOp::FuelVm(FuelVmInstruction::WideCmpOp {
                op,
                arg1: get_arg1,
                arg2: get_arg2,
            }),
        )
        .add_metadatum(context, cmp_op_metadata);

        block
            .replace_instruction(context, cmp_instr_val, wide_op, true)
            .unwrap();

        let mut additional_instrs = Vec::new();

        // lhs
        if let Some((get_lhs_local, store_lhs_local)) = lhs_store {
            additional_instrs.push(get_lhs_local);
            additional_instrs.push(store_lhs_local);
        }

        //rhs
        if let Some((get_rhs_local, store_rhs_local)) = rhs_store {
            additional_instrs.push(get_rhs_local);
            additional_instrs.push(store_rhs_local);
        }

        if arg1_needs_insert {
            additional_instrs.push(get_arg1);
        }

        if arg2_needs_insert {
            additional_instrs.push(get_arg2);
        }

        let mut inserter =
            InstructionInserter::new(context, block, crate::InsertionPosition::Before(wide_op));
        inserter.insert_slice(&additional_instrs);
    }

    Ok(true)
}

/// Find all unary operations on types bigger than 64 bits
/// and demote them to fuel specific `wide ops`, that
/// work only on pointers
fn wide_unary_op_demotion(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find all intrinsics on wide operators
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            let InstOp::UnaryOp {
                op: UnaryOpKind::Not,
                arg,
            } = instr_val.get_instruction(context)?.op
            else {
                return None;
            };

            match arg.get_type(context) {
                Some(t) if t.is_uint_of(context, 256) || t.is_b256(context) => {
                    Some((block, instr_val))
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    // Now create a local for the result
    // get ptr to each arg
    // and store the result after
    for (block, binary_op_instr_val) in candidates {
        let InstOp::UnaryOp { arg, .. } = binary_op_instr_val
            .get_instruction(context)
            .cloned()
            .unwrap()
            .op
        else {
            continue;
        };

        let unary_op_metadata = binary_op_instr_val.get_metadata(context);

        let arg_ty = arg.get_type(context).unwrap();
        let arg_metadata = arg.get_metadata(context);

        let result_local =
            function.new_unique_local_var(context, "__wide_result".to_owned(), arg_ty, None, true);
        let get_result_local =
            Value::new_instruction(context, block, InstOp::GetLocal(result_local))
                .add_metadatum(context, unary_op_metadata);
        let load_result_local =
            Value::new_instruction(context, block, InstOp::Load(get_result_local))
                .add_metadatum(context, unary_op_metadata);

        // If arg1 is not a pointer, store it to a local
        let lhs_store = arg_ty.is_ptr(context).not().then(|| {
            let lhs_local = function.new_unique_local_var(
                context,
                "__wide_lhs".to_owned(),
                arg_ty,
                None,
                false,
            );
            let get_lhs_local = Value::new_instruction(context, block, InstOp::GetLocal(lhs_local))
                .add_metadatum(context, arg_metadata);
            let store_lhs_local = Value::new_instruction(
                context,
                block,
                InstOp::Store {
                    dst_val_ptr: get_lhs_local,
                    stored_val: arg,
                },
            )
            .add_metadatum(context, arg_metadata);
            (get_lhs_local, store_lhs_local)
        });

        let (arg1_needs_insert, get_arg) = if let Some((lhs_local, _)) = &lhs_store {
            (false, *lhs_local)
        } else {
            (true, arg)
        };

        // Assert all operands are pointers
        assert!(get_arg.get_type(context).unwrap().is_ptr(context));
        assert!(get_result_local.get_type(context).unwrap().is_ptr(context));

        let wide_op = Value::new_instruction(
            context,
            block,
            InstOp::FuelVm(FuelVmInstruction::WideUnaryOp {
                op: UnaryOpKind::Not,
                arg: get_arg,
                result: get_result_local,
            }),
        )
        .add_metadatum(context, unary_op_metadata);

        block
            .replace_instruction(context, binary_op_instr_val, load_result_local, true)
            .unwrap();

        let mut additional_instrs = Vec::new();

        // lhs
        if let Some((get_lhs_local, store_lhs_local)) = lhs_store {
            additional_instrs.push(get_lhs_local);
            additional_instrs.push(store_lhs_local);
        }

        if arg1_needs_insert {
            additional_instrs.push(get_arg);
        }

        additional_instrs.push(get_result_local);
        additional_instrs.push(wide_op);

        let mut inserter = InstructionInserter::new(
            context,
            block,
            crate::InsertionPosition::Before(load_result_local),
        );
        inserter.insert_slice(&additional_instrs);
    }

    Ok(true)
}

/// Find all shift operations on types bigger than 64 bits
/// and demote them to fuel specific `wide binary ops`, that
/// work only on pointers
fn wide_shift_op_demotion(context: &mut Context, function: Function) -> Result<bool, IrError> {
    // Find all intrinsics on wide operators
    let candidates = function
        .instruction_iter(context)
        .filter_map(|(block, instr_val)| {
            let instr = instr_val.get_instruction(context)?;
            let InstOp::BinaryOp {
                op: BinaryOpKind::Lsh | BinaryOpKind::Rsh,
                arg1,
                arg2,
            } = instr.op
            else {
                return None;
            };

            let arg1_type = arg1.get_type(context);
            let arg2_type = arg2.get_type(context);

            match (arg1_type, arg2_type) {
                (Some(arg1_type), Some(arg2_type))
                    if arg1_type.is_uint_of(context, 256) && arg2_type.is_uint64(context) =>
                {
                    Some((block, instr_val))
                }
                (Some(arg1_type), Some(arg2_type))
                    if arg1_type.is_b256(context) && arg2_type.is_uint64(context) =>
                {
                    Some((block, instr_val))
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(false);
    }

    // Now create a local for the result
    // get ptr to each arg
    // and store the result after
    for (block, binary_op_instr_val) in candidates {
        let InstOp::BinaryOp { op, arg1, arg2 } = binary_op_instr_val
            .get_instruction(context)
            .cloned()
            .unwrap()
            .op
        else {
            continue;
        };

        let binary_op_metadata = binary_op_instr_val.get_metadata(context);

        let arg1_ty = arg1.get_type(context).unwrap();
        let arg1_metadata = arg1.get_metadata(context);

        let arg2_ty = arg2.get_type(context).unwrap();

        let operand_ty = arg1.get_type(context).unwrap();

        let result_local = function.new_unique_local_var(
            context,
            "__wide_result".to_owned(),
            operand_ty,
            None,
            true,
        );
        let get_result_local =
            Value::new_instruction(context, block, InstOp::GetLocal(result_local))
                .add_metadatum(context, binary_op_metadata);
        let load_result_local =
            Value::new_instruction(context, block, InstOp::Load(get_result_local))
                .add_metadatum(context, binary_op_metadata);

        // If arg1 is not a pointer, store it to a local
        let lhs_store = if !arg1_ty.is_ptr(context) {
            let lhs_local = function.new_unique_local_var(
                context,
                "__wide_lhs".to_owned(),
                operand_ty,
                None,
                false,
            );
            let get_lhs_local = Value::new_instruction(context, block, InstOp::GetLocal(lhs_local))
                .add_metadatum(context, arg1_metadata);
            let store_lhs_local = Value::new_instruction(
                context,
                block,
                InstOp::Store {
                    dst_val_ptr: get_lhs_local,
                    stored_val: arg1,
                },
            )
            .add_metadatum(context, arg1_metadata);
            Some((get_lhs_local, store_lhs_local))
        } else {
            None
        };

        let (arg1_needs_insert, get_arg1) = if let Some((lhs_local, _)) = &lhs_store {
            (false, *lhs_local)
        } else {
            (true, arg1)
        };

        // Assert result and lhs are pointers
        // Assert rhs is u64
        assert!(get_arg1.get_type(context).unwrap().is_ptr(context));
        assert!(get_result_local.get_type(context).unwrap().is_ptr(context));
        assert!(arg2_ty.is_uint64(context));

        let wide_op = Value::new_instruction(
            context,
            block,
            InstOp::FuelVm(FuelVmInstruction::WideBinaryOp {
                op,
                arg1: get_arg1,
                arg2,
                result: get_result_local,
            }),
        )
        .add_metadatum(context, binary_op_metadata);

        block
            .replace_instruction(context, binary_op_instr_val, load_result_local, true)
            .unwrap();

        let mut additional_instrs = Vec::new();

        // lhs
        if let Some((get_lhs_local, store_lhs_local)) = lhs_store {
            additional_instrs.push(get_lhs_local);
            additional_instrs.push(store_lhs_local);
        }

        if arg1_needs_insert {
            additional_instrs.push(get_arg1);
        }

        additional_instrs.push(get_result_local);
        additional_instrs.push(wide_op);

        let mut inserter = InstructionInserter::new(
            context,
            block,
            crate::InsertionPosition::Before(load_result_local),
        );
        inserter.insert_slice(&additional_instrs);
    }

    Ok(true)
}
