use crate::{
    block::Block, context::Context, error::IrError, function::Function, instruction::Instruction,
    value::ValueDatum,
};

pub fn simplify_cfg(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let mut modified = false;
    loop {
        if remove_dead_blocks(context, function)? {
            modified = true;
            continue;
        }

        if merge_blocks(context, function)? {
            modified = true;
            continue;
        }

        // Other passes here... always continue to the top if pass returns true.
        break;
    }
    Ok(modified)
}

fn remove_dead_blocks(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    // Find a block other than 'entry' which has no predecessors, remove it if found.
    function
        .block_iter(context)
        .skip(1)
        .find(|block| block.num_predecessors(context) == 0)
        .map(|dead_block| function.remove_block(context, &dead_block))
        .transpose()
        .map(|result| result.is_some())
}

fn merge_blocks(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    // Get the block a block branches to iff its terminator is `br`.
    let block_terms_with_br = |from_block: Block| -> Option<(Block, Block)> {
        from_block.get_terminator(context).and_then(|term| {
            if let Instruction::Branch(to_block) = term {
                Some((from_block, *to_block))
            } else {
                None
            }
        })
    };

    // Check whether a pair of blocks are singly paired.  i.e., `from_block` is the only
    // predecessor of `to_block`.
    let are_uniquely_paired = |(from_block, to_block): &(Block, Block)| -> bool {
        let mut preds = context.blocks[to_block.0].predecessors(context);
        preds.next() == Some(*from_block) && preds.next().is_none()
    };

    // Find a block with an unconditional branch terminator which branches to a block with that
    // single predecessor.
    let twin_blocks = function
        .block_iter(context)
        // Filter all blocks with a Branch terminator.
        .filter_map(block_terms_with_br)
        // Find branching blocks where they are singly paired.
        .find(are_uniquely_paired);

    // If not found then abort here.
    let mut block_chain = match twin_blocks {
        Some((from_block, to_block)) => vec![from_block, to_block],
        None => return Ok(false),
    };

    // There may be more blocks which are also singly paired with these twins, so iteratively
    // search for more blocks in a chain which can be all merged into one.
    loop {
        match block_terms_with_br(block_chain.last().copied().unwrap()) {
            None => {
                // There is no twin for this block.
                break;
            }
            Some(next_pair) => {
                if are_uniquely_paired(&next_pair) {
                    // Add the next `to_block` to the chain and continue.
                    block_chain.push(next_pair.1);
                } else {
                    // The chain has ended.
                    break;
                }
            }
        }
    }

    // Keep a copy of the final block in the chain so we can adjust the successors below.
    let final_to_block = block_chain.last().copied().unwrap();

    // The first block in the chain will be extended with the contents of the rest of the blocks in
    // the chain, which we'll call `from_block` since we're branching from here to the next one.
    let mut block_chain = block_chain.into_iter();
    let from_block = block_chain.next().unwrap();

    // Loop for the rest of the chain, to all the `to_block`s.
    for to_block in block_chain {
        // Replace the `phi` instruction in `to_block` with its singular element.
        let phi_val = to_block.get_phi(context);
        match &context.values[phi_val.0].value {
            ValueDatum::Instruction(Instruction::Phi(els)) if els.len() == 1 => {
                // Replace all uses of the phi and then remove it so it isn't merged below.
                let (_, ref_val) = els[0];
                function.replace_value(context, phi_val, ref_val, None);
                to_block.remove_instruction(context, phi_val);
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
    let succs = context.blocks[final_to_block.0]
        .successors(context)
        .collect::<Vec<crate::block::Block>>();
    for succ in succs {
        if let Some(phi_content) = context.values.get_mut(succ.get_phi(context).0) {
            if let ValueDatum::Instruction(Instruction::Phi(els)) = &mut phi_content.value {
                // This is the phi for a successor block of `to_block` which we're about to
                // remove/merge with `from_block`, so the predecessor needs to change from `to_block`
                // to `from_block`.
                els.iter_mut().for_each(|(block, _)| {
                    if *block == final_to_block {
                        *block = from_block;
                    }
                });
            }
        };
    }

    Ok(true)
}
