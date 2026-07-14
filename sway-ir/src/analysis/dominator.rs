use crate::{
    block::Block, AnalysisResult, AnalysisResultT, AnalysisResults, Context, Function, IrError,
    Pass, PassMutability, ScopedPass, Value,
};
use indexmap::IndexSet;
/// Dominator tree and related algorithms.
/// The algorithms implemented here are from the paper
// "A Simple, Fast Dominance Algorithm" -- Keith D. Cooper, Timothy J. Harvey, and Ken Kennedy.
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::Write;
use sway_types::{FxIndexMap, FxIndexSet};

/// Represents a node in the dominator tree.
pub struct DomTreeNode {
    /// The immediate dominator of self.
    pub parent: Option<Block>,
    /// The blocks that self immediately dominates.
    pub children: Vec<Block>,
}

impl DomTreeNode {
    pub fn new(parent: Option<Block>) -> DomTreeNode {
        DomTreeNode {
            parent,
            children: vec![],
        }
    }
}

// The dominator tree is represented by mapping each Block to its DomTreeNode.
#[derive(Default)]
pub struct DomTree(FxIndexMap<Block, DomTreeNode>);
impl AnalysisResultT for DomTree {}

// Dominance frontier sets.
pub type DomFronts = FxIndexMap<Block, FxIndexSet<Block>>;
impl AnalysisResultT for DomFronts {}

/// Post ordering of blocks in the CFG.
pub struct PostOrder {
    pub block_to_po: FxHashMap<Block, usize>,
    pub po_to_block: Vec<Block>,
}
impl AnalysisResultT for PostOrder {}

impl PostOrder {
    /// If `block` was found by the `PostOrder` analysis
    /// it is reachable from the entry function.
    #[inline(always)]
    pub fn is_reachable(&self, block: &Block) -> bool {
        self.block_to_po.contains_key(block)
    }
}

pub const POSTORDER_NAME: &str = "postorder";

pub fn create_postorder_pass() -> Pass {
    Pass {
        name: POSTORDER_NAME,
        descr: "Postorder traversal of the control-flow graph",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Analysis(compute_post_order_pass)),
    }
}

pub fn compute_post_order_pass(
    context: &Context,
    _: &AnalysisResults,
    function: Function,
) -> Result<AnalysisResult, IrError> {
    Ok(Box::new(compute_post_order(context, &function)))
}

fn post_order(
    context: &Context,
    block: Block,
    result: &mut PostOrder,
    visited: &mut FxHashSet<Block>,
    counter: &mut usize,
) {
    if !visited.insert(block) {
        return;
    }

    for successor in block.successors(context) {
        post_order(context, successor.block, result, visited, counter);
    }

    result.block_to_po.insert(block, *counter);
    result.po_to_block.push(block);
    *counter += 1;
}

/// Compute the post-order traversal of the CFG.
///
/// **BEWARE: Unreachable blocks aren't part of the result.**
pub fn compute_post_order(context: &Context, function: &Function) -> PostOrder {
    let mut result = PostOrder {
        block_to_po: FxHashMap::default(),
        po_to_block: Vec::default(),
    };

    let mut counter = 0;
    let mut visited = FxHashSet::<Block>::default();
    let entry = function.get_entry_block(context);
    post_order(context, entry, &mut result, &mut visited, &mut counter);

    // We could assert the whole thing, but it'd be expensive.
    assert!(result.po_to_block.last().unwrap() == &entry);

    result
}

pub const DOMINATORS_NAME: &str = "dominators";

pub fn create_dominators_pass() -> Pass {
    Pass {
        name: DOMINATORS_NAME,
        descr: "Dominator tree computation",
        deps: vec![POSTORDER_NAME],
        runner: ScopedPass::FunctionPass(PassMutability::Analysis(compute_dom_tree_pass)),
    }
}

// Find the nearest common dominator of two blocks,
// using the partially computed dominator tree.
fn nearest_common_dominator_of_two_blocks(
    po: &PostOrder,
    dom_tree: &FxIndexMap<Block, DomTreeNode>,
    mut block_a: Block,
    mut block_b: Block,
) -> Block {
    while block_a != block_b {
        while po.block_to_po[&block_a] < po.block_to_po[&block_b] {
            block_a = dom_tree[&block_a].parent.unwrap();
        }
        while po.block_to_po[&block_b] < po.block_to_po[&block_a] {
            block_b = dom_tree[&block_b].parent.unwrap();
        }
    }
    block_a
}

/// Compute the dominator tree for the CFG.
fn compute_dom_tree_pass(
    context: &Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<AnalysisResult, IrError> {
    let po: &PostOrder = analyses.get_analysis_result(function);
    let domtree = compute_dom_tree(context, function, po)?;
    Ok(Box::new(domtree))
}

pub(crate) fn compute_dom_tree(
    context: &Context<'_>,
    function: Function,
    po: &PostOrder,
) -> Result<DomTree, IrError> {
    let mut dom_tree = FxIndexMap::default();

    // This is to make the algorithm happy. It'll be changed to None later.
    let entry = function.get_entry_block(context);
    dom_tree.insert(entry, DomTreeNode::new(Some(entry)));

    // initialize the dominators tree. This allows us to do dom_tree[b] fearlessly.
    // Note that we just previously initialized "entry", so we skip that here.
    for block in po.po_to_block.iter().take(po.po_to_block.len() - 1) {
        dom_tree.insert(*block, DomTreeNode::new(None));
    }

    let mut changed = true;

    while changed {
        changed = false;

        // For all nodes, b, in reverse postorder (except start node)
        for block in po.po_to_block.iter().rev().skip(1) {
            let current_block_po = po.block_to_po[block];

            // new_idom <- first (processed) predecessor of `block` (pick one)
            let mut new_idom = block
                .pred_iter(context)
                .find(|pred| {
                    // "pred" may not be reachable, and hence not in the cfg.
                    let Some(pred_po) = po.block_to_po.get(pred) else {
                        return false;
                    };
                    *pred_po > current_block_po
                })
                .cloned()
                .unwrap();

            let picked_pred = new_idom;

            // for all other (reachable) predecessors, `pred` of `block`:
            // if doms[pred] already calculated
            // then new_idom is the common dominator of both
            for pred in block
                .pred_iter(context)
                .filter(|p| **p != picked_pred && po.is_reachable(p))
            {
                if dom_tree[pred].parent.is_some() {
                    new_idom =
                        nearest_common_dominator_of_two_blocks(po, &dom_tree, *pred, new_idom);
                }
            }

            // update doms[block] if needed
            let b_node = dom_tree.get_mut(block).unwrap();
            match b_node.parent {
                Some(idom) if idom == new_idom => {}
                _ => {
                    b_node.parent = Some(new_idom);
                    changed = true;
                }
            }
        }
    }

    // Fix the root.
    dom_tree.get_mut(&entry).unwrap().parent = None;

    // Build the children.
    for block in po.po_to_block.iter() {
        let Some(parent) = dom_tree[block].parent else {
            continue;
        };
        dom_tree[&parent].children.push(*block);
    }

    Ok(DomTree(dom_tree))
}

impl DomTree {
    /// Does `dominator` dominate `dominatee`?
    pub fn dominates(&self, dominator: Block, dominatee: Block) -> bool {
        let mut node_opt = Some(dominatee);
        while let Some(node) = node_opt {
            if node == dominator {
                return true;
            }
            node_opt = self.0[&node].parent;
        }
        false
    }

    /// Get an iterator over the children nodes
    pub fn children(&self, node: Block) -> impl Iterator<Item = Block> + '_ {
        self.0[&node].children.iter().cloned()
    }

    /// Get i'th child of a given node
    pub fn child(&self, node: Block, i: usize) -> Option<Block> {
        self.0[&node].children.get(i).cloned()
    }

    /// Does `dominator` dominate `dominatee`?
    pub fn dominates_instr(&self, context: &Context, dominator: Value, dominatee: Value) -> bool {
        let dominator_inst = dominator.get_instruction(context).unwrap();
        let dominatee_inst = dominatee.get_instruction(context).unwrap();

        if dominator == dominatee {
            return true;
        }
        let dominator_block = dominator_inst.parent;
        let dominatee_block = dominatee_inst.parent;
        if dominator_block == dominatee_block {
            // Same block, but different instructions.
            // Check the order of instructions in the block.
            let mut found_dominator = false;
            for instr in dominator_block.instruction_iter(context) {
                if instr == dominator {
                    found_dominator = true;
                }
                if instr == dominatee {
                    return found_dominator;
                }
            }
            false
        } else {
            self.dominates(dominator_block, dominatee_block)
        }
    }
}

pub const DOM_FRONTS_NAME: &str = "dominance-frontiers";

pub fn create_dom_fronts_pass() -> Pass {
    Pass {
        name: DOM_FRONTS_NAME,
        descr: "Dominance frontiers computation",
        deps: vec![DOMINATORS_NAME],
        runner: ScopedPass::FunctionPass(PassMutability::Analysis(compute_dom_fronts)),
    }
}

/// Compute dominance frontiers set for each block.
fn compute_dom_fronts(
    context: &Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<AnalysisResult, IrError> {
    let dom_tree: &DomTree = analyses.get_analysis_result(function);
    let mut res = DomFronts::default();
    for (b, _) in dom_tree.0.iter() {
        res.insert(*b, IndexSet::default());
    }

    // for all nodes, b
    for (b, _) in dom_tree.0.iter() {
        // if the number of predecessors of b >= 2
        if b.num_predecessors(context) > 1 {
            // unwrap() is safe as b is not "entry", and hence must have idom.
            let b_idom = dom_tree.0[b].parent.unwrap();
            // for all (reachable) predecessors, p, of b
            for p in b.pred_iter(context).filter(|&p| dom_tree.0.contains_key(p)) {
                let mut runner = *p;
                while runner != b_idom {
                    // add b to runner’s dominance frontier set
                    res.get_mut(&runner).unwrap().insert(*b);
                    runner = dom_tree.0[&runner].parent.unwrap();
                }
            }
        }
    }
    Ok(Box::new(res))
}

/// Print dominator tree in the graphviz dot format.
pub fn print_dot(context: &Context, func_name: &str, dom_tree: &DomTree) -> String {
    let mut res = format!("digraph {func_name} {{\n");
    for (b, idom) in dom_tree.0.iter() {
        if let Some(idom) = idom.parent {
            let _ = writeln!(
                res,
                "\t{} -> {}",
                idom.get_label(context),
                b.get_label(context)
            );
        }
    }
    res += "}\n";
    res
}

/// Print dominator frontiers information.
pub fn print_dom_fronts(context: &Context, func_name: &str, dom_fronts: &DomFronts) -> String {
    let mut res = format!("Dominance frontiers set for {func_name}:\n");
    for (b, dfs) in dom_fronts.iter() {
        res += &("\t".to_string() + &b.get_label(context) + ": ");
        for f in dfs {
            res += &(f.get_label(context) + " ");
        }
        res += "\n";
    }
    res
}
