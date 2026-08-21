use crate::{Context, DebugWithContext, Function, Symbol};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug)]
pub enum Cycle {
    /// Simple cycle. Examples: a <-> b, a -> c -> b
    ///                                  ˆ--------/
    Simple { nodes: Vec<Symbol> },
    /// Has a non cyclical path until the cycle:
    /// a -> b -> c -> d
    ///      ^--------/
    /// tail: [a],
    /// cycle: [b, c, d]
    /// Named like this just because visually resembles the Greek letter `rho` (p)
    RhoShape {
        tail: Vec<Symbol>,
        cycle: Vec<Symbol>,
    },
}

impl DebugWithContext for (Function, &Cycle) {
    fn fmt_with_context(
        &self,
        formatter: &mut std::fmt::Formatter,
        context: &Context,
    ) -> std::fmt::Result {
        let names = |syms: &[Symbol]| -> Vec<String> {
            syms.iter().map(|n| n.get_name(context, self.0)).collect()
        };
        match self.1 {
            Cycle::Simple { nodes } => formatter
                .debug_struct("Cycle::Simple")
                .field("nodes", &names(nodes))
                .finish(),
            Cycle::RhoShape { tail, cycle } => formatter
                .debug_struct("Cycle::RhoShape")
                .field("tail", &names(tail))
                .field("cycle", &names(cycle))
                .finish(),
        }
    }
}

impl Cycle {
    /// Categorize the found cycle. `start` must be inside the cycle.
    ///
    /// `edges` must form a partial functional graph, a directed graph where each
    /// node has at most one outgoing edge.
    pub fn new(edges: &FxHashMap<Symbol, Symbol>, start: Symbol) -> Self {
        // Starting from the `start` node, which is inside the cycle, navigate
        // forward the graph edge.
        // Because `start` is in the cycle and the graph is partially functional,
        // we know that it has a forward edge and that we will eventually come back to it.
        let cycle = std::iter::successors(Some(start), |&node| {
            let next = edges[&node];
            if next == start {
                None
            } else {
                Some(next)
            }
        })
        .collect::<Vec<_>>();

        // Now we check if this cycle has a tail, a set of nodes outside the cycle that
        // moving forward always reach the cycle.
        let cycle_set = cycle.iter().copied().collect::<FxHashSet<_>>();
        let tail = edges
            .keys()
            .filter(|node| {
                // check if is outside the cycle
                if cycle_set.contains(node) {
                    return false;
                }

                // check if moving forward always reach the cycle.
                //
                // `take(edges.len() + 1)` avoids infinite looping if the node is
                // in another cycle. This is a safe in partial functional graphs
                // because it is an upper bound for how many steps you can take
                // before you either hit a sink (node without outgoing edge) or find a cycle
                std::iter::successors(Some(*node), |current| edges.get(current))
                    .take(edges.len() + 1)
                    .any(|node| cycle_set.contains(node))
            })
            .copied()
            .collect::<Vec<_>>();

        if tail.is_empty() {
            Cycle::Simple { nodes: cycle }
        } else {
            Cycle::RhoShape { tail, cycle }
        }
    }
}

/// Find any cycle and return a `Symbol` inside of this cycle.
///
/// `edges` must form a partial functional graph, a directed graph where each
/// node has at most one outgoing edge.
pub fn find_node_in_cycle(edges: &FxHashMap<Symbol, Symbol>) -> Option<Symbol> {
    // Nodes we already know are not in a cycle
    let mut not_in_cycle: FxHashSet<Symbol> = FxHashSet::default();

    // SAFETY: Function promises ANY cycle, so the order here
    // does not matter
    #[allow(clippy::iter_over_hash_type)]
    for candidate in edges.keys() {
        if not_in_cycle.contains(candidate) {
            continue;
        }

        let mut path = FxHashSet::default();
        let mut candidate = *candidate;
        loop {
            // If we know candidate is not in a cycle,
            // all nodes leading to it, are also not in a cycle
            if not_in_cycle.contains(&candidate) {
                not_in_cycle.extend(path.drain());
                break;
            }

            // Cycle found
            if !path.insert(candidate) {
                return Some(candidate);
            }

            match edges.get(&candidate) {
                // Move forward
                Some(next) => candidate = *next,
                // Reached a sink
                None => {
                    not_in_cycle.extend(path.drain());
                    break;
                }
            }
        }
    }

    None
}
