//! Function inlining.
//!
//! Function inlining is pretty hairy so these passes must be maintained with care.

use std::{cell::RefCell, collections::HashMap};

use rustc_hash::FxHashMap;

use crate::{
    asm::AsmArg,
    block::Block,
    call_graph, compute_post_order,
    context::Context,
    error::IrError,
    function::Function,
    instruction::{FuelVmInstruction, InstOp},
    irtype::Type,
    metadata::{combine, MetadataIndex},
    value::{Value, ValueContent, ValueDatum},
    variable::LocalVar,
    AnalysisResults, BlockArgument, Instruction, Module, Pass, PassMutability, ScopedPass,
};

pub const FN_INLINE_NAME: &str = "inline";

pub fn create_fn_inline_pass() -> Pass {
    Pass {
        name: FN_INLINE_NAME,
        descr: "Function inlining",
        deps: vec![],
        runner: ScopedPass::ModulePass(PassMutability::Transform(fn_inline)),
    }
}

/// This is a copy of sway_core::inline::Inline.
/// TODO: Reuse: Depend on sway_core? Move it to sway_types?
#[derive(Debug)]
pub enum Inline {
    Always,
    Never,
}

pub fn metadata_to_inline(context: &Context, md_idx: Option<MetadataIndex>) -> Option<Inline> {
    fn for_each_md_idx<T, F: FnMut(MetadataIndex) -> Option<T>>(
        context: &Context,
        md_idx: Option<MetadataIndex>,
        mut f: F,
    ) -> Option<T> {
        // If md_idx is not None and is a list then try them all.
        md_idx.and_then(|md_idx| {
            if let Some(md_idcs) = md_idx.get_content(context).unwrap_list() {
                md_idcs.iter().find_map(|md_idx| f(*md_idx))
            } else {
                f(md_idx)
            }
        })
    }
    for_each_md_idx(context, md_idx, |md_idx| {
        // Create a new inline and save it in the cache.
        md_idx
            .get_content(context)
            .unwrap_struct("inline", 1)
            .and_then(|fields| fields[0].unwrap_string())
            .and_then(|inline_str| {
                let inline = match inline_str {
                    "always" => Some(Inline::Always),
                    "never" => Some(Inline::Never),
                    _otherwise => None,
                }?;
                Some(inline)
            })
    })
}

pub fn fn_inline(
    context: &mut Context,
    _: &AnalysisResults,
    module: Module,
) -> Result<bool, IrError> {
    // Inspect ALL calls and count how often each function is called.
    let call_counts: HashMap<Function, u64> =
        module
            .function_iter(context)
            .fold(HashMap::new(), |mut counts, func| {
                for (_block, ins) in func.instruction_iter(context) {
                    if let Some(Instruction {
                        op: InstOp::Call(callee, _args),
                        ..
                    }) = ins.get_instruction(context)
                    {
                        counts
                            .entry(*callee)
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                    }
                }
                counts
            });

    let inline_heuristic = |ctx: &Context, func: &Function, _call_site: &Value| {
        // The encoding code in the `__entry` functions contains pointer patterns that mark
        // escape analysis and referred symbols as incomplete. This effectively forbids optimizations
        // like SROA nad DCE. If we inline original entries, like e.g., `main`, the code in them will
        // also not be optimized. Therefore, we forbid inlining of original entries into `__entry`.
        if func.is_original_entry(ctx) {
            return false;
        }

        let attributed_inline = metadata_to_inline(ctx, func.get_metadata(ctx));
        match attributed_inline {
            Some(Inline::Always) => {
                // TODO: check if inlining of function is possible
                // return true;
            }
            Some(Inline::Never) => {
                return false;
            }
            None => {}
        }

        // If the function is called only once then definitely inline it.
        if call_counts.get(func).copied().unwrap_or(0) == 1 {
            return true;
        }

        // If the function is (still) small then also inline it.
        const MAX_INLINE_INSTRS_COUNT: usize = 12;
        if func.num_instructions_incl_asm_instructions(ctx) <= MAX_INLINE_INSTRS_COUNT {
            return true;
        }

        false
    };

    let cg =
        call_graph::build_call_graph(context, &module.function_iter(context).collect::<Vec<_>>());
    let functions = call_graph::callee_first_order(&cg);
    let mut modified = false;

    for function in functions {
        modified |= inline_some_function_calls(context, &function, inline_heuristic)?;
    }
    Ok(modified)
}

/// Inline all calls made from a specific function, effectively removing all `Call` instructions.
///
/// e.g., If this is applied to main() then all calls in the program are removed.  This is
/// obviously dangerous for recursive functions, in which case this pass would inline forever.
pub fn inline_all_function_calls(
    context: &mut Context,
    function: &Function,
) -> Result<bool, IrError> {
    inline_some_function_calls(context, function, |_, _, _| true)
}

/// Inline function calls based on a provided heuristic predicate.
///
/// There are many things to consider when deciding to inline a function.  For example:
/// - The size of the function, especially if smaller than the call overhead size.
/// - The stack frame size of the function.
/// - The number of calls made to the function or if the function is called inside a loop.
/// - A particular call has constant arguments implying further constant folding.
/// - An attribute request, e.g., #[always_inline], #[never_inline].
pub fn inline_some_function_calls<F: Fn(&Context, &Function, &Value) -> bool>(
    context: &mut Context,
    function: &Function,
    predicate: F,
) -> Result<bool, IrError> {
    // Find call sites which passes the predicate.
    // We use a RefCell so that the inliner can modify the value
    // when it moves other instructions (which could be in call_date) after an inline.
    let (call_sites, call_data): (Vec<_>, FxHashMap<_, _>) = function
        .instruction_iter(context)
        .filter_map(|(block, call_val)| match context.values[call_val.0].value {
            ValueDatum::Instruction(Instruction {
                op: InstOp::Call(inlined_function, _),
                ..
            }) => predicate(context, &inlined_function, &call_val).then_some((
                call_val,
                (call_val, RefCell::new((block, inlined_function))),
            )),
            _ => None,
        })
        .unzip();

    for call_site in &call_sites {
        let call_site_in = call_data.get(call_site).unwrap();
        let (block, inlined_function) = *call_site_in.borrow();

        if function == &inlined_function {
            // We can't inline a function into itself.
            continue;
        }

        inline_function_call(
            context,
            *function,
            block,
            *call_site,
            inlined_function,
            &call_data,
        )?;
    }

    Ok(!call_data.is_empty())
}

/// A utility to get a predicate which can be passed to inline_some_function_calls() based on
/// certain sizes of the function.  If a constraint is None then any size is assumed to be
/// acceptable.
///
/// The max_stack_size is a bit tricky, as the IR doesn't really know (or care) about the size of
/// types.  See the source code for how it works.
pub fn is_small_fn(
    max_blocks: Option<usize>,
    max_instrs: Option<usize>,
    max_stack_size: Option<usize>,
) -> impl Fn(&Context, &Function, &Value) -> bool {
    fn count_type_elements(context: &Context, ty: &Type) -> usize {
        // This is meant to just be a heuristic rather than be super accurate.
        if ty.is_array(context) {
            count_type_elements(context, &ty.get_array_elem_type(context).unwrap())
                * ty.get_array_len(context).unwrap() as usize
        } else if ty.is_union(context) {
            ty.get_field_types(context)
                .iter()
                .map(|ty| count_type_elements(context, ty))
                .max()
                .unwrap_or(1)
        } else if ty.is_struct(context) {
            ty.get_field_types(context)
                .iter()
                .map(|ty| count_type_elements(context, ty))
                .sum()
        } else {
            1
        }
    }

    move |context: &Context, function: &Function, _call_site: &Value| -> bool {
        max_blocks.is_none_or(|max_block_count| function.num_blocks(context) <= max_block_count)
            && max_instrs.is_none_or(|max_instrs_count| {
                function.num_instructions_incl_asm_instructions(context) <= max_instrs_count
            })
            && max_stack_size.is_none_or(|max_stack_size_count| {
                function
                    .locals_iter(context)
                    .map(|(_name, ptr)| count_type_elements(context, &ptr.get_inner_type(context)))
                    .sum::<usize>()
                    <= max_stack_size_count
            })
    }
}

/// Inline a function to a specific call site within another function.
///
/// The destination function, block and call site must be specified along with the function to
/// inline.
pub fn inline_function_call(
    context: &mut Context,
    function: Function,
    block: Block,
    call_site: Value,
    inlined_function: Function,
    call_data: &FxHashMap<Value, RefCell<(Block, Function)>>,
) -> Result<(), IrError> {
    // Split the block at right after the call site.
    let call_site_idx = block
        .instruction_iter(context)
        .position(|v| v == call_site)
        .unwrap();
    let (pre_block, post_block) = block.split_at(context, call_site_idx + 1);
    if post_block != block {
        // We need to update call_data for every call_site that was in block.
        for inst in post_block.instruction_iter(context).filter(|inst| {
            matches!(
                context.values[inst.0].value,
                ValueDatum::Instruction(Instruction {
                    op: InstOp::Call(..),
                    ..
                })
            )
        }) {
            if let Some(call_info) = call_data.get(&inst) {
                call_info.borrow_mut().0 = post_block;
            }
        }
    }

    // Remove the call from the pre_block instructions.  It's still in the context.values[] though.
    pre_block.remove_last_instruction(context);

    // Returned values, if any, go to `post_block`, so a block arg there.
    // We don't expect `post_block` to already have any block args.
    if post_block.new_arg(context, call_site.get_type(context).unwrap()) != 0 {
        panic!("Expected newly created post_block to not have block args")
    }
    function.replace_value(
        context,
        call_site,
        post_block.get_arg(context, 0).unwrap(),
        None,
    );

    // Take the locals from the inlined function and add them to this function.  `value_map` is a
    // map from the original local ptrs to the new ptrs.
    let ptr_map = function.merge_locals_from(context, inlined_function);
    let mut value_map = HashMap::new();

    // Add the mapping from argument values in the inlined function to the args passed to the call.
    if let ValueDatum::Instruction(Instruction {
        op: InstOp::Call(_, passed_vals),
        ..
    }) = &context.values[call_site.0].value
    {
        for (arg_val, passed_val) in context.functions[inlined_function.0]
            .arguments
            .iter()
            .zip(passed_vals.iter())
        {
            value_map.insert(arg_val.1, *passed_val);
        }
    }

    // Get the metadata attached to the function call which may need to be propagated to the
    // inlined instructions.
    let metadata = context.values[call_site.0].metadata;

    // Now remove the call altogether.
    context.values.remove(call_site.0);

    // Insert empty blocks from the inlined function between our split blocks, and create a mapping
    // from old blocks to new.  We need this when inlining branch instructions, so they branch to
    // the new blocks.
    //
    // We map the entry block in the inlined function (which we know must exist) to our `pre_block`
    // from the split above.  We'll start appending inlined instructions to that block rather than
    // a new one (with a redundant branch to it from the `pre_block`).
    let inlined_fn_name = inlined_function.get_name(context).to_owned();
    let mut block_map = HashMap::new();
    let mut block_iter = context.functions[inlined_function.0]
        .blocks
        .clone()
        .into_iter();
    block_map.insert(block_iter.next().unwrap(), pre_block);
    block_map = block_iter.fold(block_map, |mut block_map, inlined_block| {
        let inlined_block_label = inlined_block.get_label(context);
        let new_block = function
            .create_block_before(
                context,
                &post_block,
                Some(format!("{inlined_fn_name}_{inlined_block_label}")),
            )
            .unwrap();
        block_map.insert(inlined_block, new_block);
        // We collect so that context can be mutably borrowed later.
        let inlined_args: Vec<_> = inlined_block.arg_iter(context).copied().collect();
        for inlined_arg in inlined_args {
            if let ValueDatum::Argument(BlockArgument {
                block: _,
                idx: _,
                ty,
            }) = &context.values[inlined_arg.0].value
            {
                let index = new_block.new_arg(context, *ty);
                value_map.insert(inlined_arg, new_block.get_arg(context, index).unwrap());
            } else {
                unreachable!("Expected a block argument")
            }
        }
        block_map
    });

    // Use a reverse-post-order traversal to ensure that definitions are seen before uses.
    let inlined_block_iter = compute_post_order(context, &inlined_function)
        .po_to_block
        .into_iter()
        .rev();
    // We now have a mapping from old blocks to new (currently empty) blocks, and a mapping from
    // old values (locals and args at this stage) to new values.  We can copy instructions over,
    // translating their blocks and values to refer to the new ones.  The value map is still live
    // as we add new instructions which replace the old ones to it too.
    for ref block in inlined_block_iter {
        for ins in block.instruction_iter(context) {
            inline_instruction(
                context,
                block_map.get(block).unwrap(),
                &post_block,
                &ins,
                &block_map,
                &mut value_map,
                &ptr_map,
                metadata,
            );
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn inline_instruction(
    context: &mut Context,
    new_block: &Block,
    post_block: &Block,
    instruction: &Value,
    block_map: &HashMap<Block, Block>,
    value_map: &mut HashMap<Value, Value>,
    local_map: &HashMap<LocalVar, LocalVar>,
    fn_metadata: Option<MetadataIndex>,
) {
    // Util to translate old blocks to new.  If an old block isn't in the map then we panic, since
    // it should be guaranteed to be there...that's a bug otherwise.
    let map_block = |old_block| *block_map.get(&old_block).unwrap();

    // Util to translate old values to new.  If an old value isn't in the map then it (should be)
    // a const, which we can just keep using.
    let map_value = |old_val: Value| value_map.get(&old_val).copied().unwrap_or(old_val);
    let map_local = |old_local| local_map.get(&old_local).copied().unwrap();

    // The instruction needs to be cloned into the new block, with each value and/or block
    // translated using the above maps.  Most of these are relatively cheap as Instructions
    // generally are lightweight, except maybe ASM blocks, but we're able to re-use the block
    // content since it's a black box and not concerned with Values, Blocks or Pointers.
    //
    // We need to clone the instruction here, which is unfortunate.  Maybe in the future we
    // restructure instructions somehow, so we don't need a persistent `&Context` to access them.
    if let ValueContent {
        value: ValueDatum::Instruction(old_ins),
        metadata: val_metadata,
    } = context.values[instruction.0].clone()
    {
        // Combine the function metadata with this instruction metadata so we don't lose the
        // function metadata after inlining.
        let metadata = combine(context, &fn_metadata, &val_metadata);

        let new_ins = match old_ins.op {
            InstOp::AsmBlock(asm, args) => {
                let new_args = args
                    .iter()
                    .map(|AsmArg { name, initializer }| AsmArg {
                        name: name.clone(),
                        initializer: initializer.map(map_value),
                    })
                    .collect();

                // We can re-use the old asm block with the updated args.
                new_block.append(context).asm_block_from_asm(asm, new_args)
            }
            InstOp::BitCast(value, ty) => new_block.append(context).bitcast(map_value(value), ty),
            InstOp::UnaryOp { op, arg } => new_block.append(context).unary_op(op, map_value(arg)),
            InstOp::BinaryOp { op, arg1, arg2 } => {
                new_block
                    .append(context)
                    .binary_op(op, map_value(arg1), map_value(arg2))
            }
            // For `br` and `cbr` below we don't need to worry about the phi values, they're
            // adjusted later in `inline_function_call()`.
            InstOp::Branch(b) => new_block.append(context).branch(
                map_block(b.block),
                b.args.iter().map(|v| map_value(*v)).collect(),
            ),
            InstOp::Call(f, args) => new_block.append(context).call(
                f,
                args.iter()
                    .map(|old_val: &Value| map_value(*old_val))
                    .collect::<Vec<Value>>()
                    .as_slice(),
            ),
            InstOp::CastPtr(val, ty) => new_block.append(context).cast_ptr(map_value(val), ty),
            InstOp::Cmp(pred, lhs_value, rhs_value) => {
                new_block
                    .append(context)
                    .cmp(pred, map_value(lhs_value), map_value(rhs_value))
            }
            InstOp::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => new_block.append(context).conditional_branch(
                map_value(cond_value),
                map_block(true_block.block),
                map_block(false_block.block),
                true_block.args.iter().map(|v| map_value(*v)).collect(),
                false_block.args.iter().map(|v| map_value(*v)).collect(),
            ),
            InstOp::ContractCall {
                return_type,
                name,
                params,
                coins,
                asset_id,
                gas,
            } => new_block.append(context).contract_call(
                return_type,
                name,
                map_value(params),
                map_value(coins),
                map_value(asset_id),
                map_value(gas),
            ),
            InstOp::FuelVm(fuel_vm_instr) => match fuel_vm_instr {
                FuelVmInstruction::Gtf { index, tx_field_id } => {
                    new_block.append(context).gtf(map_value(index), tx_field_id)
                }
                FuelVmInstruction::Log {
                    log_val,
                    log_ty,
                    log_id,
                } => new_block
                    .append(context)
                    .log(map_value(log_val), log_ty, map_value(log_id)),
                FuelVmInstruction::ReadRegister(reg) => {
                    new_block.append(context).read_register(reg)
                }
                FuelVmInstruction::Revert(val) => new_block.append(context).revert(map_value(val)),
                FuelVmInstruction::JmpMem => new_block.append(context).jmp_mem(),
                FuelVmInstruction::Smo {
                    recipient,
                    message,
                    message_size,
                    coins,
                } => new_block.append(context).smo(
                    map_value(recipient),
                    map_value(message),
                    map_value(message_size),
                    map_value(coins),
                ),
                FuelVmInstruction::StateClear {
                    key,
                    number_of_slots,
                } => new_block
                    .append(context)
                    .state_clear(map_value(key), map_value(number_of_slots)),
                FuelVmInstruction::StateLoadQuadWord {
                    load_val,
                    key,
                    number_of_slots,
                } => new_block.append(context).state_load_quad_word(
                    map_value(load_val),
                    map_value(key),
                    map_value(number_of_slots),
                ),
                FuelVmInstruction::StateLoadWord(key) => {
                    new_block.append(context).state_load_word(map_value(key))
                }
                FuelVmInstruction::StateStoreQuadWord {
                    stored_val,
                    key,
                    number_of_slots,
                } => new_block.append(context).state_store_quad_word(
                    map_value(stored_val),
                    map_value(key),
                    map_value(number_of_slots),
                ),
                FuelVmInstruction::StateStoreWord { stored_val, key } => new_block
                    .append(context)
                    .state_store_word(map_value(stored_val), map_value(key)),
                FuelVmInstruction::WideUnaryOp { op, arg, result } => new_block
                    .append(context)
                    .wide_unary_op(op, map_value(arg), map_value(result)),
                FuelVmInstruction::WideBinaryOp {
                    op,
                    arg1,
                    arg2,
                    result,
                } => new_block.append(context).wide_binary_op(
                    op,
                    map_value(arg1),
                    map_value(arg2),
                    map_value(result),
                ),
                FuelVmInstruction::WideModularOp {
                    op,
                    result,
                    arg1,
                    arg2,
                    arg3,
                } => new_block.append(context).wide_modular_op(
                    op,
                    map_value(result),
                    map_value(arg1),
                    map_value(arg2),
                    map_value(arg3),
                ),
                FuelVmInstruction::WideCmpOp { op, arg1, arg2 } => new_block
                    .append(context)
                    .wide_cmp_op(op, map_value(arg1), map_value(arg2)),
                FuelVmInstruction::Retd { ptr, len } => new_block
                    .append(context)
                    .retd(map_value(ptr), map_value(len)),
            },
            InstOp::GetElemPtr {
                base,
                elem_ptr_ty,
                indices,
            } => {
                let elem_ty = elem_ptr_ty.get_pointee_type(context).unwrap();
                new_block.append(context).get_elem_ptr(
                    map_value(base),
                    elem_ty,
                    indices.iter().map(|idx| map_value(*idx)).collect(),
                )
            }
            InstOp::GetLocal(local_var) => {
                new_block.append(context).get_local(map_local(local_var))
            }
            InstOp::GetGlobal(global_var) => new_block.append(context).get_global(global_var),
            InstOp::GetStorageKey(storage_key) => new_block.append(context).get_storage_key(storage_key),
            InstOp::GetConfig(module, name) => new_block.append(context).get_config(module, name),
            InstOp::IntToPtr(value, ty) => {
                new_block.append(context).int_to_ptr(map_value(value), ty)
            }
            InstOp::Load(src_val) => new_block.append(context).load(map_value(src_val)),
            InstOp::MemCopyBytes {
                dst_val_ptr,
                src_val_ptr,
                byte_len,
            } => new_block.append(context).mem_copy_bytes(
                map_value(dst_val_ptr),
                map_value(src_val_ptr),
                byte_len,
            ),
            InstOp::MemCopyVal {
                dst_val_ptr,
                src_val_ptr,
            } => new_block
                .append(context)
                .mem_copy_val(map_value(dst_val_ptr), map_value(src_val_ptr)),
            InstOp::MemClearVal { dst_val_ptr } => new_block
                .append(context)
                .mem_clear_val(map_value(dst_val_ptr)),
            InstOp::Nop => new_block.append(context).nop(),
            InstOp::PtrToInt(value, ty) => {
                new_block.append(context).ptr_to_int(map_value(value), ty)
            }
            // We convert `ret` to `br post_block` and add the returned value as a phi value.
            InstOp::Ret(val, _) => new_block
                .append(context)
                .branch(*post_block, vec![map_value(val)]),
            InstOp::Store {
                dst_val_ptr,
                stored_val,
            } => new_block
                .append(context)
                .store(map_value(dst_val_ptr), map_value(stored_val)),
        }
        .add_metadatum(context, metadata);

        value_map.insert(*instruction, new_ins);
    }
}
