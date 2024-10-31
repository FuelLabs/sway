//! When a value is guaranteed to have a constant value in a region of the CFG,
//! this optimization replaces uses of that value with the constant in that region.

use rustc_hash::FxHashMap;

use crate::{
    AnalysisResults, Context, DomTree, Function, InstOp, Instruction, IrError, Pass,
    PassMutability, Predicate, ScopedPass, DOMINATORS_NAME,
};

pub const CCP_NAME: &str = "ccp";

pub fn create_ccp_pass() -> Pass {
    Pass {
        name: CCP_NAME,
        descr: "Conditional constant proparagion",
        deps: vec![DOMINATORS_NAME],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(ccp)),
    }
}

pub fn ccp(
    context: &mut Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let dom_tree: &DomTree = analyses.get_analysis_result(function);

    // In the set of blocks dominated by `key`, replace all uses of `val.0` with `val.1`.
    let mut dom_region_replacements = FxHashMap::default();

    for block in function.block_iter(context) {
        let term = block
            .get_terminator(context)
            .expect("Malformed block: no terminator");
        if let InstOp::ConditionalBranch {
            cond_value,
            true_block,
            false_block: _,
        } = &term.op
        {
            if let Some(Instruction {
                parent: _,
                op: InstOp::Cmp(pred, v1, v2),
            }) = cond_value.get_instruction(context)
            {
                if matches!(pred, Predicate::Equal)
                    && (v1.is_constant(context) ^ v2.is_constant(context)
                        && true_block.block.num_predecessors(context) == 1)
                {
                    if v1.is_constant(context) {
                        dom_region_replacements.insert(true_block.block, (*v2, *v1));
                    } else {
                        dom_region_replacements.insert(true_block.block, (*v1, *v2));
                    }
                }
            }
        }
    }

    // lets walk the dominator tree from the root.
    let root_block = function.get_entry_block(context);

    if dom_region_replacements.is_empty() {
        return Ok(false);
    }

    let mut stack = vec![(root_block, 0)];
    let mut replacements = FxHashMap::default();
    while let Some((block, next_child)) = stack.last().cloned() {
        let cur_replacement_opt = dom_region_replacements.get(&block);

        if next_child == 0 {
            // Preorder processing
            if let Some(cur_replacement) = cur_replacement_opt {
                replacements.insert(cur_replacement.0, cur_replacement.1);
            }
            // walk the current block.
            block.replace_values(context, &replacements);
        }

        // walk children.
        if let Some(child) = dom_tree.child(block, next_child) {
            // When we arrive back at "block" next time, we should process the next child.
            stack.last_mut().unwrap().1 = next_child + 1;
            // Go on to process the child.
            stack.push((child, 0));
        } else {
            // No children left to process. Start postorder processing.
            if let Some(cur_replacement) = cur_replacement_opt {
                replacements.remove(&cur_replacement.0);
            }
            stack.pop();
        }
    }

    Ok(true)
}
