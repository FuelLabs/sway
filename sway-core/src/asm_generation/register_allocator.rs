use crate::asm_generation::{
    register_sequencer::RegisterSequencer, RegisterAllocationStatus, RegisterPool,
};
use crate::asm_lang::{virtual_register::*, RealizedOp, VirtualOp};
use petgraph::graph::NodeIndex;
use std::collections::{BTreeSet, HashMap};

pub type InterferenceGraph =
    petgraph::stable_graph::StableGraph<VirtualRegister, (), petgraph::Undirected>;

/// Given a list of instructions `ops` of a program, do liveness analysis for the full program.
///
/// A virtual registers is live at some point in the program if it has previously been defined by
/// an instruction and will be used by an instruction in the future.
///
/// The analysis function below assumes that it is possible that a virtual register is assigned
/// more than once. That is, it doesn't assume that the intermediate assembly is in SSA form.
///
/// Two tables are generated: `live_in` and `live_out`. Each row in the tables corresponds to an
/// instruction in the program.
/// * A virtual register is in the `live_out` table for a given instruction if it is live on any
/// of that node's out-edges
/// * A virtual register is in the `live_in` table for a given instruction if it is live on any
/// of that node's in-edges
///
///
/// Algorithm:
/// ===============================================================================================
/// for each instruction op:
///     live_in(op) = {}
///     live_out(op) = {}
///     def(op) = list of virtual registers defined by op
///     use(op) = list of virtual registers used by op
///
/// repeat
///     for each instruction op (traversed in reverse topological order of the CFG)
///         prev_live_in(op) = live_in(op)
///         prev_live_out(op) = live_out(op)
///         live_out(op) = live_in(s_1) UNIONl ive_in(s_2) UNION live_in(s_3) UNION ...
///                        where s_1, s_2, s_3, ... are all the successors of op in the CFG.
///         live_in(op) = use(op) UNION (live_out(op) - def(op))
/// until     prev_live_in(op) = live_in(op)
///       AND prev_live_out(op) = live_out(op)
/// ===============================================================================================
///
/// Note that we're only looking at registers that have the enum variant
/// VirtualRegister::Virtual(_). All other registers (i.e. ones with the
/// VirtualRegister::Constant(_) variant) are assumed to be live throughout the full program.
///
/// This function finally returns `live_out` because it has all the liveness information needed.
/// `live_in` is computed because it is needed to compute `live_out` iteratively.
///
pub(crate) fn liveness_analysis(ops: &[RealizedOp]) -> HashMap<usize, BTreeSet<VirtualRegister>> {
    // Hash maps that will reprsent the live_in and live_out tables. The key of each hash map is
    // simply the index of each instruction in the `ops` vector.
    let mut live_in: HashMap<usize, BTreeSet<VirtualRegister>> = HashMap::new();
    let mut live_out: HashMap<usize, BTreeSet<VirtualRegister>> = HashMap::new();
    for i in 0..ops.len() {
        live_in.insert(i, BTreeSet::new());
        live_out.insert(i, BTreeSet::new());
    }

    // Simple mapping between the actual offset of an instruction and its index in the `ops`
    // vector.
    let mut offset_to_ix: HashMap<u64, usize> = HashMap::new();
    for (ix, op) in ops.iter().enumerate() {
        offset_to_ix.insert(op.offset, ix);
    }

    let mut modified: bool;
    while {
        modified = false;
        // Iterate in reverse topological order of the CFG (which is basically the same as the
        // reverse order of `ops`. This makes the outer `while` loop converge faster.
        for (ix, op) in ops.iter().rev().enumerate() {
            let rev_ix = ops.len() - ix - 1;

            // Get use and def vectors without any of the Constant registers
            let mut op_use = op.opcode.use_registers();
            let mut op_def = op.opcode.def_registers();
            op_use.retain(|&reg| matches!(reg, VirtualRegister::Virtual(_)));
            op_def.retain(|&reg| matches!(reg, VirtualRegister::Virtual(_)));

            let prev_live_out_op = live_out.get(&rev_ix).expect("ix must exist").clone();
            let prev_live_in_op = live_in.get(&rev_ix).expect("ix must exist").clone();

            // Compute live_out(op) = live_in(s_1) UNION live_in(s_2) UNION ..., where s1, s_2, ...
            // are successors of op
            let live_out_op = live_out.get_mut(&rev_ix).expect("ix must exist");
            for s in &op.opcode.successors(rev_ix, ops, &offset_to_ix) {
                for l in live_in.get(s).expect("ix must exist") {
                    live_out_op.insert(l.clone());
                }
            }

            // Compute live_in(op) = use(op) UNION (live_out(op) - def(op))
            // Add use(op)
            let live_in_op = live_in.get_mut(&rev_ix).expect("ix must exist");
            for u in op_use {
                live_in_op.insert(u.clone());
            }

            // Add live_out(op) - def(op)
            let mut live_out_op_minus_defs = live_out_op.clone();
            for d in &op_def {
                live_out_op_minus_defs.remove(d);
            }
            for l in &live_out_op_minus_defs {
                live_in_op.insert(l.clone());
            }

            // Did anything change in this iteration?
            modified = (prev_live_in_op != *live_in_op) || (prev_live_out_op != *live_out_op);
        }
        modified
    } {}

    live_out
}

/// Given a list of instructions `ops` and a `live_out` table computed using the method
/// `liveness_analysis()`, create an interference graph (aka a "conflict" graph):
/// * Nodes = virtual registers
/// * Edges = overlapping live ranges
///
/// Two virtual registers interfere if there exists a point in the program where both are
/// simultaneously live. If `v1` and `v2` interfere, they cannot be allocated to the same register.
///
/// Algorithm:
/// ===============================================================================================
/// 1. create a graph node for every virtual register used.
/// 2. for a MOVE "v <= c" with live_out virtual registers b1, ... bn for v:
///        add edges (v, b_1), ..., (v, b_n) for any b_i different from c.
/// 3. for non-MOVE def of virtual register v with live_out virtual registers b_1, ..., b_n:
///        add edges (v, b_1), ..., (v, b_n)
/// ===============================================================================================
///
pub(crate) fn create_interference_graph(
    ops: &[RealizedOp],
    live_out: &HashMap<usize, BTreeSet<VirtualRegister>>,
) -> (InterferenceGraph, HashMap<VirtualRegister, NodeIndex>) {
    let mut interference_graph = InterferenceGraph::with_capacity(0, 0);

    // Figure out a mapping between a given VirtualRegister and its corresponding NodeIndex
    // in the interference graph.
    let mut reg_to_node_map: HashMap<VirtualRegister, NodeIndex> = HashMap::new();

    // Get all virtual registers used by the intermediate assembly and add them to the graph
    ops.iter()
        .fold(BTreeSet::new(), |mut tree, elem| {
            let mut regs = elem.opcode.registers();
            regs.retain(|&reg| matches!(reg, VirtualRegister::Virtual(_)));
            tree.extend(regs.into_iter());
            tree
        })
        .iter()
        .for_each(|&reg| {
            reg_to_node_map.insert(reg.clone(), interference_graph.add_node(reg.clone()));
        });

    for (ix, regs) in live_out {
        match &ops[*ix].opcode {
            VirtualOp::MOVE(v, c) => {
                if let Some(ix1) = reg_to_node_map.get(v) {
                    for b in regs.iter() {
                        if let Some(ix2) = reg_to_node_map.get(b) {
                            // Add edge (v, b) if b != c
                            // Also, avoid adding self edges and edges that already exist
                            if *b != *c && *b != *v && !interference_graph.contains_edge(*ix1, *ix2)
                            {
                                interference_graph.add_edge(*ix1, *ix2, ());
                            }
                        }
                    }
                }
            }
            _ => {
                for v in &ops[*ix].opcode.def_registers() {
                    if let Some(ix1) = reg_to_node_map.get(v) {
                        for b in regs.iter() {
                            if let Some(ix2) = reg_to_node_map.get(b) {
                                // Add edge (v, b)
                                // Avoid adding self edges and edges that already exist
                                if *b != **v && !interference_graph.contains_edge(*ix1, *ix2) {
                                    interference_graph.add_edge(*ix1, *ix2, ());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (interference_graph, reg_to_node_map)
}

/// Given a list of instructions `ops` and a corresponding interference_graph, generate a new,
/// smaller list of instructions, where unnecessary MOVE instructions have been removed. When an
/// unnecessary MOVE is detected and removed, the two virtual registers used by the MOVE are said
/// to be "coalesced" and the two corresponding nodes in the graph are then merged.
///
/// Two important aspects of this for our implementation:
/// * When two registers are coalesced, a new node with a new virtual register (generated using the
///   register sequencer) is created in the interference graph.
/// * When a MOVE instruction is removed, the offset of each subsequent instruction has to be
///   updated, as well as the immediate values for some or all jump instructions (`ji` and `jnei`
///   for now).
///
pub(crate) fn coalesce_registers(
    ops: &[RealizedOp],
    interference_graph: &mut InterferenceGraph,
    reg_to_node_map: &mut HashMap<VirtualRegister, NodeIndex>,
    register_sequencer: &mut RegisterSequencer,
) -> Vec<RealizedOp> {
    // A map from the virtual registers that are removed to the virtual registers that they are
    // replaced with during the coalescing process.
    let mut reg_to_reg_map: HashMap<VirtualRegister, VirtualRegister> = HashMap::new();

    // To hold the final *reduced* list of ops
    let mut reduced_ops: Vec<RealizedOp> = vec![];

    // To figure out a mapping between the old offset and the new offset for each instruction. Will
    // help determine the new "immediate values" for jump instructions.
    let mut offset_map: HashMap<u64, u64> = HashMap::new();
    let mut num_moves_removed = 0;

    for op in ops {
        let new_op = RealizedOp {
            opcode: op.opcode.clone(),
            owning_span: op.owning_span.clone(),
            comment: op.comment.clone(),
            offset: op.offset - num_moves_removed,
        };
        offset_map.insert(op.offset, op.offset - num_moves_removed);
        match &op.opcode {
            VirtualOp::MOVE(x, y) => {
                match (x, y) {
                    (VirtualRegister::Virtual(_), VirtualRegister::Virtual(_)) => {
                        // Use reg_to_reg_map to figure out what x and y have been replaced
                        // with. We keep looking for mappings within reg_to_reg_map until we find a
                        // register that doesn't map to any other.
                        let regs = vec![x.clone(), y.clone()]
                            .iter()
                            .map(|reg| {
                                let mut temp = reg.clone();
                                while let Some(t) = reg_to_reg_map.get(&temp) {
                                    temp = t.clone();
                                }
                                temp
                            })
                            .collect::<Vec<_>>();
                        let (r1, r2) = (&regs[0], &regs[1]);

                        // Find the interference graph nodes that corresponding to r1 and r2
                        let ix1 = reg_to_node_map.get(r1).unwrap();
                        let ix2 = reg_to_node_map.get(r2).unwrap();

                        // If r1 and r2 are the same, the MOVE instruction can be safely removed,
                        // i.e., not added to reduced_ops
                        if r1 == r2 {
                            num_moves_removed += 1;
                            continue;
                        }

                        // If r1 and r2 are connected in the interference graph (i.e. their
                        // respective liveness ranges overalp), preserve the MOVE instruction by
                        // adding it to reduced_ops
                        if interference_graph.contains_edge(*ix1, *ix2) {
                            reduced_ops.push(new_op);
                            continue;
                        }

                        // The MOVE instruction can now be safely removed. That is, we simply don't
                        // add it to the reduced_ops vector. Also, we combine the two nodes ix1 and
                        // ix2 in the graph by creating a new node that inherits the edges of both
                        // ix1 and ix2, and then we remove ix1 and ix2 from the graph. We also have
                        // to do some bookkeeping.
                        //
                        // Note that because the interference graph is of type StableGraph, the
                        // node index corresponding to each virtual register does not change when
                        // some graph nodes are added or removed.

                        // Create a new virtual register to represent the result of coalescing of
                        // r1 and r2. Then create a node for it in the graph
                        let new_reg = register_sequencer.next();
                        let new_ix = interference_graph.add_node(new_reg.clone());

                        // Add all of ix1(r1)'s edges
                        for neighbor in interference_graph.neighbors(*ix1).collect::<Vec<_>>() {
                            interference_graph.add_edge(neighbor, new_ix, ());
                        }

                        // Add all of ix2(r2)'s edges
                        for neighbor in interference_graph.neighbors(*ix2).collect::<Vec<_>>() {
                            if !interference_graph.contains_edge(neighbor, new_ix) {
                                interference_graph.add_edge(neighbor, new_ix, ());
                            }
                        }

                        // Now remove ix1(r1) and ix2(r2)
                        interference_graph.remove_node(*ix1);
                        interference_graph.remove_node(*ix2);

                        // Update the register maps
                        reg_to_node_map.insert(new_reg.clone(), new_ix);
                        reg_to_node_map.insert(r1.clone(), new_ix);
                        reg_to_node_map.insert(r2.clone(), new_ix);
                        reg_to_reg_map.insert(r1.clone(), new_reg.clone());
                        reg_to_reg_map.insert(r2.clone(), new_reg.clone());

                        num_moves_removed += 1;
                    }
                    _ => {
                        // Preserve the MOVE instruction if either registers used in the MOVE is
                        // special registers (i.e. *not* a VirtualRegister::Virtual(_))
                        reduced_ops.push(new_op);
                    }
                }
            }
            _ => {
                // Preserve all other instructions
                reduced_ops.push(new_op);
            }
        }
    }

    // Update immediate values for jump instructions using offset_map
    for new_op in &mut reduced_ops {
        new_op.opcode = new_op.opcode.update_jump_immediate_values(&offset_map);
    }

    // Create a *final* reg-to-reg map that We keep looking for mappings within reg_to_reg_map
    // until we find a register that doesn't map to any other.
    let mut final_reg_to_reg_map: HashMap<VirtualRegister, VirtualRegister> = HashMap::new();
    for reg in reg_to_reg_map.keys() {
        let mut temp = reg;
        while let Some(t) = reg_to_reg_map.get(temp) {
            temp = t;
        }
        final_reg_to_reg_map.insert(reg.clone(), temp.clone());
    }

    // Update the registers for all instructions using final_reg_to_reg_map
    for new_op in &mut reduced_ops {
        new_op.opcode = new_op.opcode.update_register(&final_reg_to_reg_map);
    }

    reduced_ops
}

/// Given an interference graph and a integer k, figure out if the graph k-colorable. Graph
/// coloring is an NP-complete problem, but the algorithm below is a simple stack based
/// approximation that relies on the fact that any node n in the graph that has fewer than k
/// neighbors can always be colored.
///
/// Algorithm:
/// ===============================================================================================
/// 1. Pick any node n such that degree(n) < k and put it on the stack along with its neighbors.
/// 2. Remove node n and all its edges from the graph
///    - This may make some new nodes have fewer than k neighbours which is nice.
/// 3. If some vertex n still has k or more neighbors, then the graph is not k colorable, and we
///    have to spill. For now, we will just error out, but we may be able to do better in the
///    future.
/// ===============================================================================================
///
pub(crate) fn color_interference_graph(
    interference_graph: &mut InterferenceGraph,
    k: u8,
) -> Vec<(VirtualRegister, BTreeSet<VirtualRegister>)> {
    let mut stack: Vec<(VirtualRegister, BTreeSet<VirtualRegister>)> = vec![];

    while let Some(node) = pick_node(interference_graph, k) {
        let neighbors = interference_graph
            .neighbors(node)
            .map(|n| interference_graph[n].clone())
            .collect();
        stack.push((
            interference_graph
                .remove_node(node)
                .expect("Node must exist"),
            neighbors,
        ));
    }

    // If any nodes are left in the graph, then the must still have a degree larger than k. In this
    // case, the graph is not colorable.
    if interference_graph.node_count() > 0 {
        unimplemented!(
            "The allocator cannot resolve a register mapping for this program. 
                This is a temporary artifact of the extremely early stage version of this language. 
                Try to lower the number of variables you use."
        );
    }

    stack
}

/// Use the stack generated by the coloring algorithm to figure out a register assignment for each
/// virtual register. The idea here is to successively pop the stack while selecting a register to
/// each virtual register. A register r is available to a virtual register v if the intersection of
/// the neighbors of v (available from the stack) and the list of virtual registers already used by
/// r (available in the used_by field) is empty.
///
pub(crate) fn assign_registers(
    stack: &mut Vec<(VirtualRegister, BTreeSet<VirtualRegister>)>,
) -> RegisterPool {
    let mut pool = RegisterPool::init();
    while let Some((reg, neighbors)) = stack.pop() {
        if matches!(reg, VirtualRegister::Virtual(_)) {
            let available =
                pool.registers
                    .iter_mut()
                    .find(|RegisterAllocationStatus { reg: _, used_by }| {
                        neighbors.intersection(used_by).count() == 0
                    });

            if let Some(RegisterAllocationStatus { reg: _, used_by }) = available {
                used_by.insert(reg.clone());
            }
        }
    }

    pool
}

/// Picks a node from the graph that has degree less than k
pub(crate) fn pick_node(interference_graph: &InterferenceGraph, k: u8) -> Option<NodeIndex> {
    for n in interference_graph.node_indices() {
        if let VirtualRegister::Virtual(_) = interference_graph[n] {
            if interference_graph.neighbors(n).count() < k as usize {
                return Some(n);
            }
        }
    }
    None
}
