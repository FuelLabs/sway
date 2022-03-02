//! Function inlining.
//!
//! Function inlining is pretty hairy so these passes must be maintained with care.

use std::collections::HashMap;

use crate::{
    asm::AsmArg,
    block::Block,
    context::Context,
    error::IrError,
    function::Function,
    instruction::Instruction,
    pointer::Pointer,
    value::{Value, ValueContent, ValueDatum},
};

/// Inline all calls made from a specific function, effectively removing all `Call` instructions.
///
/// e.g., If this is applied to main() then all calls in the program are removed.  This is
/// obviously dangerous for recursive functions, in which case this pass would inline forever.
pub fn inline_all_function_calls(
    context: &mut Context,
    function: &Function,
) -> Result<bool, IrError> {
    let mut modified = false;
    loop {
        // Find the next call site.
        let call_data = function
            .instruction_iter(context)
            .find_map(|(block, call_val)| match context.values[call_val.0].value {
                ValueDatum::Instruction(Instruction::Call(inlined_function, _)) => {
                    Some((block, call_val, inlined_function))
                }
                _ => None,
            });
        match call_data {
            Some((block, call_val, inlined_function)) => {
                inline_function_call(context, *function, block, call_val, inlined_function)?;
                modified = true;
            }
            None => break,
        }
    }
    Ok(modified)
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
) -> Result<(), IrError> {
    // Split the block at right after the call site.
    let call_site_idx = context.blocks[block.0]
        .instructions
        .iter()
        .position(|&v| v == call_site)
        .unwrap();
    let (pre_block, post_block) = block.split_at(context, call_site_idx + 1);

    // Remove the call from the pre_block instructions.  It's still in the context.values[] though.
    context.blocks[pre_block.0].instructions.pop();

    // Replace any reference to the call with the `phi` in `post_block` since it'll now receive the
    // old return value from the inlined function.
    function.replace_value(
        context,
        call_site,
        post_block.get_phi(context),
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
        block_map
    });

    // We now have a mapping from old blocks to new (currently empty) blocks, and a mapping from
    // old values (locals and args at this stage) to new values.  We can copy instructions over,
    // translating their blocks and values to refer to the new ones.  The value map is still live
    // as we add new instructions which replace the old ones to it too.
    //
    // Note: inline_instruction() doesn't translate `phi` instructions here.
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
            );
        }
    }

    // Now we can go through and update the `phi` instructions.  We need to clone the instruction
    // here, which is unfortunate.  Maybe in the future we restructure instructions somehow, so we
    // don't need a peristent `&Context` to access them.
    for old_block in inlined_blocks {
        let new_block = block_map.get(&old_block).unwrap();
        let old_phi_val = old_block.get_phi(context);
        if let ValueDatum::Instruction(Instruction::Phi(pairs)) =
            context.values[old_phi_val.0].value.clone()
        {
            for (from_block, phi_value) in pairs {
                new_block.add_phi(
                    context,
                    block_map.get(&from_block).copied().unwrap(),
                    value_map.get(&phi_value).copied().unwrap_or(phi_value),
                );
            }
        }
    }

    Ok(())
}

fn inline_instruction(
    context: &mut Context,
    new_block: &Block,
    post_block: &Block,
    instruction: &Value,
    block_map: &HashMap<Block, Block>,
    value_map: &mut HashMap<Value, Value>,
    ptr_map: &HashMap<Pointer, Pointer>,
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
        span_md_idx,
    } = context.values[instruction.0].clone()
    {
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
                new_block
                    .ins(context)
                    .asm_block_from_asm(asm, new_args, span_md_idx)
            }
            // For `br` and `cbr` below we don't need to worry about the phi values, they're
            // adjusted later in `inline_function_call()`.
            Instruction::Branch(b) => {
                new_block
                    .ins(context)
                    .branch(map_block(b), None, span_md_idx)
            }
            Instruction::Call(f, args) => new_block.ins(context).call(
                f,
                args.iter()
                    .map(|old_val: &Value| map_value(*old_val))
                    .collect::<Vec<Value>>()
                    .as_slice(),
                span_md_idx,
            ),
            Instruction::ConditionalBranch {
                cond_value,
                true_block,
                false_block,
            } => new_block.ins(context).conditional_branch(
                map_value(cond_value),
                map_block(true_block),
                map_block(false_block),
                None,
                span_md_idx,
            ),
            Instruction::ExtractElement {
                array,
                ty,
                index_val,
            } => new_block.ins(context).extract_element(
                map_value(array),
                ty,
                map_value(index_val),
                span_md_idx,
            ),
            Instruction::ExtractValue {
                aggregate,
                ty,
                indices,
            } => {
                new_block
                    .ins(context)
                    .extract_value(map_value(aggregate), ty, indices, span_md_idx)
            }
            Instruction::GetPointer(ptr) => {
                new_block.ins(context).get_ptr(map_ptr(ptr), span_md_idx)
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
                span_md_idx,
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
                span_md_idx,
            ),
            Instruction::Load(ptr) => new_block.ins(context).load(map_ptr(ptr), span_md_idx),
            Instruction::Nop => new_block.ins(context).nop(),
            // We convert `ret` to `br post_block` and add the returned value as a phi value.
            Instruction::Ret(val, _) => {
                new_block
                    .ins(context)
                    .branch(*post_block, Some(map_value(val)), span_md_idx)
            }
            Instruction::Store { ptr, stored_val } => {
                new_block
                    .ins(context)
                    .store(map_ptr(ptr), map_value(stored_val), span_md_idx)
            }

            // NOTE: We're not translating the phi value yet, since this is the single instance of
            // use of a value which may not be mapped yet -- a branch from a subsequent block,
            // back up to this block.  And we don't need to add a `phi` instruction because an
            // empty one is added upon block creation; we can return that instead.
            Instruction::Phi(_) => new_block.get_phi(context),
        };
        value_map.insert(*instruction, new_ins);
    }
}
