//! ## Simplify Control Flow Graph
//!
//! The optimizations here aim to reduce the complexity in control flow by removing basic blocks.
//! This may be done by removing 'dead' blocks which are no longer called (or in other words, have
//! no predecessors) or by merging blocks which are linked by a single unconditional branch.
//!
//! Removing blocks will make the IR neater and more efficient but will also remove indirection of
//! data flow via PHI instructions which in turn can make analyses for passes like constant folding
//! much simpler.

use crate::{
    block::Block, context::Context, error::IrError, function::Function, instruction::Instruction,
    value::ValueDatum,
};

pub fn simplify_cfg(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let mut modified = false;
    modified |= remove_dead_blocks(context, function)?;

    loop {
        if merge_blocks(context, function)? {
            modified = true;
            continue;
        }
        break;
    }

    modified |= unlink_empty_blocks(context, function)?;

    Ok(modified)
}

fn unlink_empty_blocks(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let mut modified = false;
    let candidates: Vec<_> = function
        .block_iter(context)
        .skip(1)
        .filter_map(|block| {
            match block.get_terminator(context) {
                // Except for a branch, we don't want anything else.
                Some(Instruction::Branch(to_block)) if block.num_instructions(context) <= 1 => {
                    Some((block, to_block.clone()))
                }
                _ => None,
            }
        })
        .collect();
    for (block, (to_block, cur_params)) in candidates {
        // If `to_block`'s predecessors and `block`'s predecessors intersect,
        // AND `to_block` has an arg, then we have that pred branching to to_block
        // with different args. While that's valid IR, it's harder to generate
        // ASM for it, so let's just skip that for now.
        if to_block.num_args(context) > 0
            && to_block.pred_iter(context).any(|to_block_pred| {
                block
                    .pred_iter(context)
                    .any(|block_pred| block_pred == to_block_pred)
            })
        {
            // We cannot filter this out in candidates itself because this condition
            // may get updated *during* this optimization (i.e., inside this loop).
            continue;
        }
        let preds: Vec<_> = block.pred_iter(context).copied().collect();
        for pred in preds {
            // Whatever parameters "block" passed to "to_block", that
            // should now go from "pred" to "to_block".
            let params_from_pred = pred.get_succ_params(context, &block);
            let new_params = cur_params
                .iter()
                .map(|cur_param| match &context.values[cur_param.0].value {
                    ValueDatum::Argument(arg) => {
                        // An argument should map to the actual parameter passed.
                        params_from_pred[arg.idx]
                    }
                    _ => *cur_param,
                })
                .collect();

            pred.replace_successor(context, block, to_block, new_params);
            modified = true;
        }
    }
    Ok(modified)
}

fn remove_dead_blocks(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let mut worklist = Vec::<Block>::new();
    let mut reachable = std::collections::HashSet::<Block>::new();

    // The entry is always reachable. Let's begin with that.
    let entry_block = function.get_entry_block(context);
    reachable.insert(entry_block);
    worklist.push(entry_block);

    // Mark reachable nodes.
    while !worklist.is_empty() {
        let block = worklist.pop().unwrap();
        let succs = block.successors(context);
        for (succ, _) in succs {
            // If this isn't already marked reachable, we mark it and add to the worklist.
            if !reachable.contains(&succ) {
                reachable.insert(succ);
                worklist.push(succ);
            }
        }
    }

    // Delete all unreachable nodes.
    let mut modified = false;
    for block in function.block_iter(context) {
        if !reachable.contains(&block) {
            modified = true;

            for (succ, _) in block.successors(context) {
                succ.remove_pred(context, &block);
            }

            function.remove_block(context, &block)?;
        }
    }

    Ok(modified)
}

fn merge_blocks(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    // Check if block branches soley to another block B, and that B has exactly one predecessor.
    let check_candidate = |from_block: Block| -> Option<(Block, Block)> {
        from_block
            .get_terminator(context)
            .and_then(|term| match term {
                Instruction::Branch((to_block, _)) if to_block.num_predecessors(context) == 1 => {
                    Some((from_block, *to_block))
                }
                _ => None,
            })
    };

    // Find a block with an unconditional branch terminator which branches to a block with that
    // single predecessor.
    let twin_blocks = function
        .block_iter(context)
        // Find our candidate.
        .find_map(check_candidate);

    // If not found then abort here.
    let mut block_chain = match twin_blocks {
        Some((from_block, to_block)) => vec![from_block, to_block],
        None => return Ok(false),
    };

    // There may be more blocks which are also singly paired with these twins, so iteratively
    // search for more blocks in a chain which can be all merged into one.
    loop {
        match check_candidate(block_chain.last().copied().unwrap()) {
            None => {
                // There is no twin for this block.
                break;
            }
            Some(next_pair) => {
                block_chain.push(next_pair.1);
            }
        }
    }

    // Keep a copy of the final block in the chain so we can adjust the successors below.
    let final_to_block = block_chain.last().copied().unwrap();
    let final_to_block_succs = final_to_block.successors(context);

    // The first block in the chain will be extended with the contents of the rest of the blocks in
    // the chain, which we'll call `from_block` since we're branching from here to the next one.
    let mut block_chain = block_chain.into_iter();
    let from_block = block_chain.next().unwrap();

    // Loop for the rest of the chain, to all the `to_block`s.
    for to_block in block_chain {
        let from_params = from_block.get_succ_params(context, &to_block);
        // We collect here so that we can have &mut Context later on.
        let to_blocks: Vec<_> = to_block.arg_iter(context).copied().enumerate().collect();
        for (arg_idx, to_block_arg) in to_blocks {
            // replace all uses of `to_block_arg` with the parameter from `from_block`.
            function.replace_value(context, to_block_arg, from_params[arg_idx], None);
        }

        // Re-get the block contents mutably.
        let (from_contents, to_contents) = context.blocks.get2_mut(from_block.0, to_block.0);
        let from_contents = from_contents.unwrap();
        let to_contents = to_contents.unwrap();

        // Drop the terminator from `from_block`.
        from_contents.instructions.pop();

        // Move instructions from `to_block` to `from_block`.
        from_contents
            .instructions
            .append(&mut to_contents.instructions);

        // Remove `to_block`.
        function.remove_block(context, &to_block)?;
    }

    // Adjust the successors to the final `to_block` to now be successors of the fully merged
    // `from_block`.
    for (succ, _) in final_to_block_succs {
        succ.replace_pred(context, &final_to_block, &from_block)
    }

    Ok(true)
}
