/// Build call graphs for the program being compiled.
/// If a function F1 calls function F2, then the call
/// graph has an edge F1->F2.
use crate::{Context, Function, InstOp, Instruction, ValueDatum};

use indexmap::IndexSet;
use sway_types::{FxIndexMap, FxIndexSet};

pub type CallGraph = FxIndexMap<Function, FxIndexSet<Function>>;

/// Build call graph considering all providing functions.
pub fn build_call_graph(ctx: &Context, functions: &[Function]) -> CallGraph {
    let mut res = CallGraph::default();
    for function in functions {
        let entry = res.entry(*function);
        let entry = entry.or_insert_with(IndexSet::default);
        for (_, inst) in function.instruction_iter(ctx) {
            if let ValueDatum::Instruction(Instruction {
                op: InstOp::Call(callee, _),
                ..
            }) = ctx.values[inst.0].value
            {
                entry.insert(callee);
            }
        }
    }
    res
}

/// Given a call graph, return reverse topological sort
/// (post order traversal), i.e., If A calls B, then B
/// occurs before A in the returned Vec.
pub fn callee_first_order(cg: &CallGraph) -> Vec<Function> {
    let mut res = Vec::new();

    let mut visited = FxIndexSet::<Function>::default();
    fn post_order_visitor(
        cg: &CallGraph,
        visited: &mut FxIndexSet<Function>,
        res: &mut Vec<Function>,
        node: Function,
    ) {
        if visited.contains(&node) {
            return;
        }
        visited.insert(node);
        for callee in &cg[&node] {
            post_order_visitor(cg, visited, res, *callee);
        }
        res.push(node);
    }
    for node in cg.keys() {
        post_order_visitor(cg, &mut visited, &mut res, *node);
    }

    res
}
