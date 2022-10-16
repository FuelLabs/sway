use crate::{block::Block, BranchToWithArgs, Context, Function};
/// Dominator tree and related algorithms.
/// The algorithms implemented here are from the paper
// "A Simple, Fast Dominance Algorithm" -- Keith D. Cooper, Timothy J. Harvey, and Ken Kennedy.
use rustc_hash::{FxHashMap, FxHashSet};

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
pub type DomTree = FxHashMap<Block, DomTreeNode>;
// Dominance frontier sets.
pub type DomFronts = FxHashMap<Block, FxHashSet<Block>>;

/// Post ordering of blocks in the CFG.
pub struct PostOrder {
    pub block_to_po: FxHashMap<Block, usize>,
    pub po_to_block: Vec<Block>,
}

/// Compute the post-order traversal of the CFG.
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
    assert!(res.po_to_block[function.num_blocks(context) - 1] == entry);

    res
}

/// Compute the dominator tree for the CFG.
pub fn compute_dom_tree(context: &Context, function: &Function) -> DomTree {
    let po = compute_post_order(context, function);
    let mut dom_tree = DomTree::default();
    let entry = function.get_entry_block(context);

    // This is to make the algorithm happy. It'll be changed to None later.
    dom_tree.insert(entry, DomTreeNode::new(Some(entry)));
    // initialize the dominators tree. This allows us to do dom_tree[b] fearlessly.
    // Note that we just previously initialized "entry", so we skip that here.
    for b in po.po_to_block.iter().take(function.num_blocks(context) - 1) {
        dom_tree.insert(*b, DomTreeNode::new(None));
    }
    let mut changed = true;

    while changed {
        changed = false;
        // For all nodes, b, in reverse postorder (except start node)
        for b in po.po_to_block.iter().rev().skip(1) {
            // new_idom <- first (processed) predecessor of b (pick one)
            let mut new_idom = b
                .pred_iter(context)
                .find(|p| po.block_to_po[p] > po.block_to_po[b])
                .cloned()
                .unwrap();
            let picked_pred = new_idom;
            // for all other predecessors, p, of b:
            for p in b.pred_iter(context).filter(|p| **p != picked_pred) {
                match dom_tree[p].parent {
                    Some(_) => {
                        // if doms[p] already calculated
                        new_idom = intersect(&po, &mut dom_tree, *p, new_idom);
                    }
                    None => (),
                }
            }
            let b_node = dom_tree.get_mut(b).unwrap();
            match b_node.parent {
                Some(idom) if idom == new_idom => {}
                _ => {
                    b_node.parent = Some(new_idom);
                    changed = true;
                }
            }
        }
    }

    fn intersect(
        po: &PostOrder,
        dom_tree: &mut DomTree,
        mut finger1: Block,
        mut finger2: Block,
    ) -> Block {
        while finger1 != finger2 {
            while po.block_to_po[&finger1] < po.block_to_po[&finger2] {
                finger1 = dom_tree[&finger1].parent.unwrap();
            }
            while po.block_to_po[&finger2] < po.block_to_po[&finger1] {
                finger2 = dom_tree[&finger2].parent.unwrap();
            }
        }
        finger1
    }

    // Fix the root.
    dom_tree.get_mut(&entry).unwrap().parent = None;
    // Build the children.
    for (b, idom) in dom_tree.iter_mut() {
        idom.children.push(*b);
    }

    dom_tree
}

/// Compute dominance frontiers set for each block.
pub fn compute_dom_fronts(context: &Context, function: &Function, dom_tree: &DomTree) -> DomFronts {
    let mut res = DomFronts::default();
    for b in function.block_iter(context) {
        res.insert(b, FxHashSet::default());
    }

    // for all nodes, b
    for b in function.block_iter(context) {
        // if the number of predecessors of b >= 2
        if b.num_predecessors(context) > 1 {
            // unwrap() is safe as b is not "entry", and hence must have idom.
            let b_idom = dom_tree[&b].parent.unwrap();
            // for all predecessors, p, of b
            for p in b.pred_iter(context) {
                let mut runner = *p;
                while runner != b_idom {
                    // add b to runner’s dominance frontier set
                    res.get_mut(&runner).unwrap().insert(b);
                    runner = dom_tree[&runner].parent.unwrap();
                }
            }
        }
    }
    res
}

/// Print dominator tree in the graphviz dot format.
pub fn print_dot(context: &Context, func_name: &str, dom_tree: &DomTree) -> String {
    let mut res = format!("digraph {} {{\n", func_name);
    for (b, idom) in dom_tree.iter() {
        match idom.parent {
            Some(idom) => {
                res += &(format!(
                    "\t{} -> {}\n",
                    idom.get_label(context),
                    b.get_label(context)
                ))
            }
            None => (),
        }
    }
    res += "}\n";
    res
}

/// Print dominator frontiers information.
pub fn print_dom_fronts(context: &Context, func_name: &str, dom_fronts: &DomFronts) -> String {
    let mut res = format!("Dominance frontiers set for {}:\n", func_name);
    for (b, dfs) in dom_fronts.iter() {
        res += &("\t".to_string() + &b.get_label(context) + ": ");
        for f in dfs {
            res += &(f.get_label(context) + " ");
        }
        res += "\n";
    }
    res
}
