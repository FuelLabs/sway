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

use std::collections::HashMap;

pub fn simplify_cfg(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let mut modified = false;
    modified |= remove_dead_blocks(context, function)?;

    let pred_counts = function.count_predecessors(context);
    loop {
        if merge_blocks(context, &pred_counts, function)? {
            modified = true;
            continue;
        }
        break;
    }
    Ok(modified)
}

fn remove_dead_blocks(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let mut worklist = std::vec::Vec::<Block>::new();
    let mut reachable = std::collections::HashSet::<Block>::new();

    // The entry is always reachable. Let's begin with that.
    let entry_block = function.get_entry_block(context);
    reachable.insert(entry_block);
    worklist.push(entry_block);

    // Mark reachable nodes.
    while !worklist.is_empty() {
        let block = worklist.pop().unwrap();
        let succs = block.successors(context);
        for succ in succs {
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
            function.remove_block(context, &block)?;
        }
    }

    Ok(modified)
}

fn merge_blocks(
    context: &mut Context,
    pred_counts: &HashMap<Block, usize>,
    function: &Function,
) -> Result<bool, IrError> {
    // Check if block branches soley to another block B, and that B has exactly one predecessor.
    let check_candidate = |from_block: Block| -> Option<(Block, Block)> {
        from_block
            .get_terminator(context)
            .and_then(|term| match term {
                Instruction::Branch(to_block) if pred_counts[to_block] == 1 => {
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
        // Replace the `phi` instruction in `to_block` with its singular element.
        let phi_val = to_block.get_phi(context);
        match &context.values[phi_val.0].value {
            ValueDatum::Instruction(Instruction::Phi(els)) if els.len() <= 1 => {
                // Replace all uses of the phi and then remove it so it isn't merged below.
                if let Some((_, ref_val)) = els.get(0) {
                    function.replace_value(context, phi_val, *ref_val, None);
                    to_block.remove_instruction(context, phi_val);
                }
            }
            _otherwise => return Err(IrError::InvalidPhi),
        };

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
    for succ in final_to_block_succs {
        succ.update_phi_source_block(context, final_to_block, from_block)
    }

    Ok(true)
}
