use crate::{
    block::Block, AnalysisResult, AnalysisResultT, AnalysisResults, BranchToWithArgs, Context,
    Function, IrError, Pass, PassMutability, ScopedPass, Value,
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

/// Compute the post-order traversal of the CFG.
/// Beware: Unreachable blocks aren't part of the result.
pub fn compute_post_order(context: &Context, function: &Function) -> PostOrder {
    let mut res = PostOrder {
        block_to_po: FxHashMap::default(),
        po_to_block: Vec::default(),
    };
    let entry = function.get_entry_block(context);

    let mut counter = 0;
    let mut on_stack = FxHashSet::<Block>::default();
    fn post_order(
        context: &Context,
        n: Block,
        res: &mut PostOrder,
        on_stack: &mut FxHashSet<Block>,
        counter: &mut usize,
    ) {
        if on_stack.contains(&n) {
            return;
        }
        on_stack.insert(n);
        for BranchToWithArgs { block: n_succ, .. } in n.successors(context) {
            post_order(context, n_succ, res, on_stack, counter);
        }
        res.block_to_po.insert(n, *counter);
        res.po_to_block.push(n);
        *counter += 1;
    }
    post_order(context, entry, &mut res, &mut on_stack, &mut counter);

    // We could assert the whole thing, but it'd be expensive.
    assert!(res.po_to_block.last().unwrap() == &entry);

    res
}

pub const DOMINATORS_NAME: &str = "dominators";

pub fn create_dominators_pass() -> Pass {
    Pass {
        name: DOMINATORS_NAME,
        descr: "Dominator tree computation",
        deps: vec![POSTORDER_NAME],
        runner: ScopedPass::FunctionPass(PassMutability::Analysis(compute_dom_tree)),
    }
}

/// Compute the dominator tree for the CFG.
fn compute_dom_tree(
    context: &Context,
    analyses: &AnalysisResults,
    function: Function,
) -> Result<AnalysisResult, IrError> {
    let po: &PostOrder = analyses.get_analysis_result(function);
    let mut dom_tree = DomTree::default();
    let entry = function.get_entry_block(context);

    // This is to make the algorithm happy. It'll be changed to None later.
    dom_tree.0.insert(entry, DomTreeNode::new(Some(entry)));
    // initialize the dominators tree. This allows us to do dom_tree[b] fearlessly.
    // Note that we just previously initialized "entry", so we skip that here.
    for b in po.po_to_block.iter().take(po.po_to_block.len() - 1) {
        dom_tree.0.insert(*b, DomTreeNode::new(None));
    }
    let mut changed = true;

    while changed {
        changed = false;
        // For all nodes, b, in reverse postorder (except start node)
        for b in po.po_to_block.iter().rev().skip(1) {
            // new_idom <- first (processed) predecessor of b (pick one)
            let mut new_idom = b
                .pred_iter(context)
                .find(|p| {
                    // "p" may not be reachable, and hence not in dom_tree.
                    po.block_to_po
                        .get(p)
                        .is_some_and(|p_po| *p_po > po.block_to_po[b])
                })
                .cloned()
                .unwrap();
            let picked_pred = new_idom;
            // for all other (reachable) predecessors, p, of b:
            for p in b
                .pred_iter(context)
                .filter(|p| **p != picked_pred && po.block_to_po.contains_key(p))
            {
                if dom_tree.0[p].parent.is_some() {
                    // if doms[p] already calculated
                    new_idom = intersect(po, &dom_tree, *p, new_idom);
                }
            }
            let b_node = dom_tree.0.get_mut(b).unwrap();
            match b_node.parent {
                Some(idom) if idom == new_idom => {}
                _ => {
                    b_node.parent = Some(new_idom);
                    changed = true;
                }
            }
        }
    }

    // Find the nearest common dominator of two blocks,
    // using the partially computed dominator tree.
    fn intersect(
        po: &PostOrder,
        dom_tree: &DomTree,
        mut finger1: Block,
        mut finger2: Block,
    ) -> Block {
        while finger1 != finger2 {
            while po.block_to_po[&finger1] < po.block_to_po[&finger2] {
                finger1 = dom_tree.0[&finger1].parent.unwrap();
            }
            while po.block_to_po[&finger2] < po.block_to_po[&finger1] {
                finger2 = dom_tree.0[&finger2].parent.unwrap();
            }
        }
        finger1
    }

    // Fix the root.
    dom_tree.0.get_mut(&entry).unwrap().parent = None;
    // Build the children.
    let child_parent: Vec<_> = dom_tree
        .0
        .iter()
        .filter_map(|(n, n_node)| n_node.parent.map(|n_parent| (*n, n_parent)))
        .collect();
    for (child, parent) in child_parent {
        dom_tree.0.get_mut(&parent).unwrap().children.push(child);
    }

    Ok(Box::new(dom_tree))
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
