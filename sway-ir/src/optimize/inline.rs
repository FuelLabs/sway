//! Function inlining.
//!
//! Function inlining is pretty hairy so these passes must be maintained with care.

use std::{cell::RefCell, collections::HashMap};

use rustc_hash::FxHashMap;

use crate::{
    asm::AsmArg,
    block::Block,
    context::Context,
    error::IrError,
    function::Function,
    instruction::Instruction,
    irtype::Type,
    metadata::{combine, MetadataIndex},
    pointer::Pointer,
    value::{Value, ValueContent, ValueDatum},
    BlockArgument,
};

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
    let call_data: FxHashMap<Value, RefCell<(Block, Function)>> = function
        .instruction_iter(context)
        .filter_map(|(block, call_val)| match context.values[call_val.0].value {
            ValueDatum::Instruction(Instruction::Call(inlined_function, _)) => {
                predicate(context, &inlined_function, &call_val)
                    .then_some((call_val, RefCell::new((block, inlined_function))))
            }
            _ => None,
        })
        .collect();

    for (call_site, call_site_in) in &call_data {
        let (block, inlined_function) = *call_site_in.borrow();
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
        match ty {
            Type::Unit
            | Type::Bool
            | Type::Uint(_)
            | Type::B256
            | Type::String(_)
            | Type::Pointer(_)
            | Type::Slice => 1,
            Type::Array(aggregate) => {
                let (ty, sz) = context.aggregates[aggregate.0].array_type();
                count_type_elements(context, ty) * *sz as usize
            }
            Type::Union(aggregate) => context.aggregates[aggregate.0]
                .field_types()
                .iter()
                .map(|ty| count_type_elements(context, ty))
                .max()
                .unwrap_or(1),
            Type::Struct(aggregate) => context.aggregates[aggregate.0]
                .field_types()
                .iter()
                .map(|ty| count_type_elements(context, ty))
                .sum(),
        }
    }

    move |context: &Context, function: &Function, _call_site: &Value| -> bool {
        max_blocks
            .map(|max_block_count| function.num_blocks(context) <= max_block_count)
            .unwrap_or(true)
            && max_instrs
                .map(|max_instrs_count| function.num_instructions(context) <= max_instrs_count)
                .unwrap_or(true)
            && max_stack_size
                .map(|max_stack_size_count| {
                    function
                        .locals_iter(context)
                        .map(|(_name, ptr)| count_type_elements(context, ptr.get_type(context)))
                        .sum::<usize>()
                        <= max_stack_size_count
                })
                .unwrap_or(true)
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
    let call_site_idx = context.blocks[block.0]
        .instructions
        .iter()
        .position(|&v| v == call_site)
        .unwrap();
    let (pre_block, post_block) = block.split_at(context, call_site_idx + 1);
    if post_block != block {
        // We need to update call_data for every call_site that was in block.
        for inst in post_block.instruction_iter(context).filter(|inst| {
            matches!(
                context.values[inst.0].value,
                ValueDatum::Instruction(Instruction::Call(..))
            )
        }) {
            if let Some(call_info) = call_data.get(&inst) {
                call_info.borrow_mut().0 = post_block;
            }
        }
    }

    // Remove the call from the pre_block instructions.  It's still in the context.values[] though.
    context.blocks[pre_block.0].instructions.pop();

    // Returned values, if any, go to `post_block`, so a block arg there.
    // We don't expect `post_block` to already have any block args.
    if post_block.new_arg(context, call_site.get_type(context).unwrap()) != 0 {
        panic!("Expected newly created post_block to not have block args")
    }
    function.replace_value(
        context,
        call_site,
        post_block.get_arg(context, 0).unwrap(),
        Some(post_block),
    );

    // Take the locals from the inlined function and add them to this function.  `value_map` is a
    // map from the original local ptrs to the new ptrs.
    let ptr_map = function.merge_locals_from(context, inlined_function)?;
    let mut value_map = HashMap::new();

    // Add the mapping from argument values in the inlined function to the args passed to the call.
    if let ValueDatum::Instruction(Instruction::Call(_, passed_vals)) =
        &context.values[call_site.0].value
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
                Some(format!("{}_{}", inlined_fn_name, inlined_block_label)),
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

    // We now have a mapping from old blocks to new (currently empty) blocks, and a mapping from
    // old values (locals and args at this stage) to new values.  We can copy instructions over,
    // translating their blocks and values to refer to the new ones.  The value map is still live
    // as we add new instructions which replace the old ones to it too.
    let inlined_blocks = context.functions[inlined_function.0].blocks.clone();
    for block in &inlined_blocks {
        for ins in context.blocks[block.0].instructions.clone() {
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
    ptr_map: &HashMap<Pointer, Pointer>,
    fn_metadata: Option<MetadataIndex>,
) {
    // Util to translate old blocks to new.  If an old block isn't in the map then we panic, since
    // it should be guaranteed to be there...that's a bug otherwise.
    let map_block = |old_block| *block_map.get(&old_block).unwrap();

    // Util to translate old values to new.  If an old value isn't in the map then it (should be)
    // a const, which we can just keep using.
    let map_value = |old_val: Value| value_map.get(&old_val).copied().unwrap_or(old_val);
    let map_ptr = |old_ptr| ptr_map.get(&old_ptr).copied().unwrap();

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

        let new_ins = match old_ins {
            Instruction::AsmBlock(asm, args) => {
                let new_args = args
                    .iter()
                    .map(|AsmArg { name, initializer }| AsmArg {
                        name: name.clone(),
                        initializer: initializer.map(&map_value),
                    })
                    .collect();

                // We can re-use the old asm block with the updated args.
                new_block.ins(context).asm_block_from_asm(asm, new_args)
            }
            Instruction::AddrOf(arg) => new_block.ins(context).addr_of(map_value(arg)),
            Instruction::BitCast(value, ty) => new_block.ins(context).bitcast(map_value(value), ty),
            Instruction::BinaryOp { op, arg1, arg2 } => {
                new_block
                    .ins(context)
                    .binary_op(op, map_value(arg1), map_value(arg2))
            }
            // For `br` and `cbr` below we don't need to worry about the phi values, they're
            // adjusted later in `inline_function_call()`.
            Instruction::Branch(b) => new_block.ins(context).branch(
                map_block(b.block),
                b.args.iter().map(|v| map_value(*v)).collect(),
            ),
            Instruction::Call(f, args) => new_block.ins(context).call(
                f,
                args.iter()
                    .map(|old_val: &Value| map_value(*old_val))
                    .collect::<Vec<Value>>()
                    .as_slice(),
            ),
            Instruction::Cmp(pred, lhs_value, rhs_value) => {
                new_block
                    .ins(context)
                    .cmp(pred, map_value(lhs_value), map_value(rhs_value))
            }
            Instruction::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => new_block.ins(context).conditional_branch(
                map_value(cond_value),
                map_block(true_block.block),
                map_block(false_block.block),
                true_block.args.iter().map(|v| map_value(*v)).collect(),
                false_block.args.iter().map(|v| map_value(*v)).collect(),
            ),
            Instruction::ContractCall {
                return_type,
                name,
                params,
                coins,
                asset_id,
                gas,
            } => new_block.ins(context).contract_call(
                return_type,
                name,
                map_value(params),
                map_value(coins),
                map_value(asset_id),
                map_value(gas),
            ),
            Instruction::ExtractElement {
                array,
                ty,
                index_val,
            } => new_block
                .ins(context)
                .extract_element(map_value(array), ty, map_value(index_val)),
            Instruction::ExtractValue {
                aggregate,
                ty,
                indices,
            } => new_block
                .ins(context)
                .extract_value(map_value(aggregate), ty, indices),
            Instruction::GetStorageKey => new_block.ins(context).get_storage_key(),
            Instruction::GetPointer {
                base_ptr,
                ptr_ty,
                offset,
            } => {
                let ty = *ptr_ty.get_type(context);
                new_block
                    .ins(context)
                    .get_ptr(map_ptr(base_ptr), ty, offset)
            }
            Instruction::Gtf { index, tx_field_id } => {
                new_block.ins(context).gtf(map_value(index), tx_field_id)
            }
            Instruction::InsertElement {
                array,
                ty,
                value,
                index_val,
            } => new_block.ins(context).insert_element(
                map_value(array),
                ty,
                map_value(value),
                map_value(index_val),
            ),
            Instruction::InsertValue {
                aggregate,
                ty,
                value,
                indices,
            } => new_block.ins(context).insert_value(
                map_value(aggregate),
                ty,
                map_value(value),
                indices,
            ),
            Instruction::IntToPtr(value, ty) => {
                new_block.ins(context).int_to_ptr(map_value(value), ty)
            }
            Instruction::Load(src_val) => new_block.ins(context).load(map_value(src_val)),
            Instruction::Log {
                log_val,
                log_ty,
                log_id,
            } => new_block
                .ins(context)
                .log(map_value(log_val), log_ty, map_value(log_id)),
            Instruction::MemCopy {
                dst_val,
                src_val,
                byte_len,
            } => new_block
                .ins(context)
                .mem_copy(map_value(dst_val), map_value(src_val), byte_len),
            Instruction::Nop => new_block.ins(context).nop(),
            Instruction::ReadRegister(reg) => new_block.ins(context).read_register(reg),
            // We convert `ret` to `br post_block` and add the returned value as a phi value.
            Instruction::Ret(val, _) => new_block
                .ins(context)
                .branch(*post_block, vec![map_value(val)]),
            Instruction::Revert(val) => new_block.ins(context).revert(map_value(val)),
            Instruction::StateLoadQuadWord { load_val, key } => new_block
                .ins(context)
                .state_load_quad_word(map_value(load_val), map_value(key)),
            Instruction::StateLoadWord(key) => {
                new_block.ins(context).state_load_word(map_value(key))
            }
            Instruction::StateStoreQuadWord { stored_val, key } => new_block
                .ins(context)
                .state_store_quad_word(map_value(stored_val), map_value(key)),
            Instruction::StateStoreWord { stored_val, key } => new_block
                .ins(context)
                .state_store_word(map_value(stored_val), map_value(key)),
            Instruction::Store {
                dst_val,
                stored_val,
            } => new_block
                .ins(context)
                .store(map_value(dst_val), map_value(stored_val)),
        }
        .add_metadatum(context, metadata);

        value_map.insert(*instruction, new_ins);
    }
}
