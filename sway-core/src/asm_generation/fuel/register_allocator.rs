use crate::{
    asm_generation::fuel::{analyses::liveness_analysis, compiler_constants},
    asm_lang::{
        allocated_ops::AllocatedRegister, virtual_register::*, AllocatedAbstractOp, Op,
        VirtualImmediate12, VirtualImmediate18, VirtualImmediate24, VirtualOp,
    },
};

use either::Either;
use indexmap::IndexMap;
use petgraph::{
    stable_graph::NodeIndex,
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Ordering;
use std::collections::{hash_map, BTreeSet, HashMap};
use sway_error::error::CompileError;
use sway_ir::size_bytes_round_up_to_word_alignment;
use sway_types::{FxIndexSet, Span};

// Each node in the interference graph represents a VirtualRegister.
// An edge from V1 -> V2 means that V2 was an open live range at the
// the time V1 was defined. For spilling, incoming edges matter more
// as it indicates how big the range is, and thus is better to spill.
// An edge has a "bool" weight to indicate whether it was deleted
// during colouring. We don't actually delete the edge because that's
// required again during the actual assignment.
pub type InterferenceGraph =
    petgraph::stable_graph::StableGraph<VirtualRegister, bool, petgraph::Directed>;

// Initially, the bytecode will have a lot of individual registers being used. Each register will
// have a new unique identifier. For example, two separate invocations of `+` will result in 4
// registers being used for arguments and 2 for outputs.
//
// After that, the level 0 bytecode will go through a process where register use is minified,
// producing level 1 bytecode. This process is as such:
//
// 1. Detect the last time a register is read. After that, it can be reused and recycled to fit the
//    needs of the next "level 0 bytecode" register
//
// 2. Detect needless assignments and movements, and substitute registers in.
//    i.e.
//    a = b
//    c = a
//
//    would become
//    c = b
//
//
// After the level 1 bytecode is produced, level 2 bytecode is created by limiting the maximum
// number of registers and inserting bytecode to read from/write to memory where needed. Ideally,
// the algorithm for determining which registers will be written off to memory is based on how
// frequently that register is accessed in a particular section of code. Using this strategy, we
// hope to minimize memory writing.
//
// For each line, the number of times a virtual register is accessed between then and the end of the
// program is its register precedence. A virtual register's precedence is 0 if it is currently in
// "memory", and the above described number if it is not. This prevents over-prioritization of
// registers that have already been written off to memory.
//
/// The [SwayAsmSet] contains either a contract ABI and corresponding ASM, a script's main
/// function's ASM, or a predicate's main function's ASM. ASM is never generated for libraries,
/// as that happens when the library itself is imported.

#[derive(Debug)]
struct RegisterAllocationStatus {
    reg: AllocatedRegister,
    used_by: BTreeSet<VirtualRegister>,
}

#[derive(Debug)]
pub(crate) struct RegisterPool {
    registers: Vec<RegisterAllocationStatus>,
}

impl RegisterPool {
    fn init() -> Self {
        let reg_pool: Vec<RegisterAllocationStatus> = (0
                // - 1 because we reserve the final register for the data_section begin
                ..compiler_constants::NUM_ALLOCATABLE_REGISTERS)
            .map(|x| RegisterAllocationStatus {
                reg: AllocatedRegister::Allocated(x),
                used_by: BTreeSet::new(),
            })
            .collect();
        Self {
            registers: reg_pool,
        }
    }

    pub(crate) fn get_register(
        &self,
        virtual_register: &VirtualRegister,
    ) -> Option<AllocatedRegister> {
        let allocated_reg =
            self.registers
                .iter()
                .find(|RegisterAllocationStatus { reg: _, used_by }| {
                    used_by.contains(virtual_register)
                });

        allocated_reg.map(|RegisterAllocationStatus { reg, used_by: _ }| reg.clone())
    }
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
///    add edges (v, b_1), ..., (v, b_n) for any b_i different from c.
/// 3. for non-MOVE def of virtual register v with live_out virtual registers b_1, ..., b_n:
///    add edges (v, b_1), ..., (v, b_n)
///
/// ===============================================================================================
pub(crate) fn create_interference_graph(
    ops: &[Op],
    live_out: &[BTreeSet<VirtualRegister>],
) -> (InterferenceGraph, HashMap<VirtualRegister, NodeIndex>) {
    let mut interference_graph = InterferenceGraph::with_capacity(0, 0);

    // Figure out a mapping between a given VirtualRegister and its corresponding NodeIndex
    // in the interference graph.
    let mut reg_to_node_map: HashMap<VirtualRegister, NodeIndex> = HashMap::new();

    // Get all virtual registers used by the intermediate assembly and add them to the graph
    ops.iter()
        .fold(BTreeSet::new(), |mut tree, elem| {
            let mut regs = elem.registers();
            regs.retain(|&reg| reg.is_virtual());
            tree.extend(regs);
            tree
        })
        .iter()
        .for_each(|&reg| {
            reg_to_node_map.insert(reg.clone(), interference_graph.add_node(reg.clone()));
        });

    for (ix, regs) in live_out.iter().enumerate() {
        match &ops[ix].opcode {
            Either::Left(VirtualOp::MOVE(v, c)) => {
                if let Some(ix1) = reg_to_node_map.get(v) {
                    for b in regs.iter() {
                        if let Some(ix2) = reg_to_node_map.get(b) {
                            // Add edge (v, b) if b != c
                            // Also, avoid adding self edges
                            if *b != *c && *b != *v {
                                interference_graph.update_edge(*ix1, *ix2, true);
                            }
                        }
                    }
                }
            }
            _ => {
                for v in &ops[ix].def_registers() {
                    if let Some(ix1) = reg_to_node_map.get(v) {
                        for b in regs.iter() {
                            if let Some(ix2) = reg_to_node_map.get(b) {
                                // Add edge (v, b)
                                // Avoid adding self edges
                                if *b != **v {
                                    interference_graph.update_edge(*ix1, *ix2, true);
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
///   updated, as well as the immediate values for some or all jump instructions (`ji`, `jnei`, and
///   `jnzi for now).
///
pub(crate) fn coalesce_registers(
    ops: &[Op],
    live_out: Vec<BTreeSet<VirtualRegister>>,
    interference_graph: &mut InterferenceGraph,
    reg_to_node_map: &mut HashMap<VirtualRegister, NodeIndex>,
) -> (Vec<Op>, Vec<BTreeSet<VirtualRegister>>) {
    // A map from the virtual registers that are removed to the virtual registers that they are
    // replaced with during the coalescing process.
    let mut reg_to_reg_map = IndexMap::<&VirtualRegister, &VirtualRegister>::new();

    // To hold the final *reduced* list of ops
    let mut reduced_ops: Vec<Op> = Vec::with_capacity(ops.len());
    let mut reduced_live_out: Vec<BTreeSet<VirtualRegister>> = Vec::with_capacity(live_out.len());
    assert!(ops.len() == live_out.len());

    for (op_idx, op) in ops.iter().enumerate() {
        match &op.opcode {
            Either::Left(VirtualOp::MOVE(x, y)) => {
                match (x, y) {
                    (VirtualRegister::Virtual(_), VirtualRegister::Virtual(_)) => {
                        // Use reg_to_reg_map to figure out what x and y have been replaced
                        // with. We keep looking for mappings within reg_to_reg_map until we find a
                        // register that doesn't map to any other.
                        let mut r1 = x;
                        while let Some(t) = reg_to_reg_map.get(r1) {
                            r1 = t;
                        }
                        let mut r2 = y;
                        while let Some(t) = reg_to_reg_map.get(r2) {
                            r2 = t;
                        }

                        // Find the interference graph nodes that corresponding to r1 and r2
                        let ix1 = reg_to_node_map.get(r1).unwrap();
                        let ix2 = reg_to_node_map.get(r2).unwrap();

                        // If r1 and r2 are the same, the MOVE instruction can be safely removed,
                        // i.e., not added to reduced_ops
                        if r1 == r2 {
                            continue;
                        }

                        let r1_neighbours = interference_graph
                            .neighbors_undirected(*ix1)
                            .collect::<FxIndexSet<_>>();
                        let r2_neighbours = interference_graph
                            .neighbors_undirected(*ix2)
                            .collect::<FxIndexSet<_>>();

                        // Using either of the two safety conditions below, it's guaranteed
                        // that we aren't turning a k-colourable graph into one that's not,
                        // by doing the coalescing. Ref: "Coalescing" section in Appel's book.
                        let briggs_safety = r1_neighbours
                            .union(&r2_neighbours)
                            .filter(|&&neighbour| {
                                interference_graph.neighbors_undirected(neighbour).count()
                                    >= compiler_constants::NUM_ALLOCATABLE_REGISTERS as usize
                            })
                            .count()
                            < compiler_constants::NUM_ALLOCATABLE_REGISTERS as usize;

                        let george_safety = r2_neighbours.iter().all(|&r2_neighbor| {
                            r1_neighbours.contains(&r2_neighbor)
                                || interference_graph.neighbors_undirected(r2_neighbor).count()
                                    < compiler_constants::NUM_ALLOCATABLE_REGISTERS as usize
                        });

                        let safe = briggs_safety || george_safety;

                        // If r1 and r2 are connected in the interference graph (i.e. their
                        // respective liveness ranges overalp), preserve the MOVE instruction by
                        // adding it to reduced_ops
                        if interference_graph.contains_edge(*ix1, *ix2)
                            || interference_graph.contains_edge(*ix2, *ix1)
                            || !safe
                        {
                            reduced_ops.push(op.clone());
                            reduced_live_out.push(live_out[op_idx].clone());
                            continue;
                        }

                        // The MOVE instruction can now be safely removed. That is, we simply don't
                        // add it to the reduced_ops vector. Also, we combine the two nodes ix1 and
                        // ix2 into ix2 and then we remove ix1 from the graph. We also have
                        // to do some bookkeeping.
                        //
                        // Note that because the interference graph is of type StableGraph, the
                        // node index corresponding to each virtual register does not change when
                        // some graph nodes are added or removed.

                        // Add all of ix1(r1)'s edges to `ix2(r2)` as incoming edges.
                        for neighbor in r1_neighbours {
                            if !interference_graph.contains_edge(*ix2, neighbor) {
                                interference_graph.update_edge(neighbor, *ix2, true);
                            }
                        }

                        // Remove ix1.
                        interference_graph.remove_node(*ix1);

                        // Update the register maps
                        reg_to_node_map.insert(r1.clone(), *ix2);
                        reg_to_reg_map.insert(r1, r2);
                    }
                    _ => {
                        // Preserve the MOVE instruction if either registers used in the MOVE is
                        // special registers (i.e. *not* a VirtualRegister::Virtual(_))
                        reduced_ops.push(op.clone());
                        reduced_live_out.push(live_out[op_idx].clone());
                    }
                }
            }
            _ => {
                // Preserve all other instructions
                reduced_ops.push(op.clone());
                reduced_live_out.push(live_out[op_idx].clone());
            }
        }
    }

    // Create a *final* reg-to-reg map that We keep looking for mappings within reg_to_reg_map
    // until we find a register that doesn't map to any other.
    let mut final_reg_to_reg_map = IndexMap::<&VirtualRegister, &VirtualRegister>::new();
    for reg in reg_to_reg_map.keys() {
        let mut temp = reg;
        while let Some(t) = reg_to_reg_map.get(temp) {
            temp = t;
        }
        final_reg_to_reg_map.insert(reg, temp);
    }

    // Update the registers for all instructions using final_reg_to_reg_map
    for new_op in &mut reduced_ops {
        *new_op = new_op.update_register(&final_reg_to_reg_map);
    }
    for new_live_out in &mut reduced_live_out {
        for (old, &new) in &final_reg_to_reg_map {
            if new_live_out.remove(old) {
                new_live_out.insert(new.clone());
            }
        }
    }

    (reduced_ops, reduced_live_out)
}

// For every virtual register, compute its (def points, use points).
fn compute_def_use_points(ops: &[Op]) -> FxHashMap<VirtualRegister, (Vec<usize>, Vec<usize>)> {
    let mut res: FxHashMap<VirtualRegister, (Vec<usize>, Vec<usize>)> = FxHashMap::default();
    for (idx, op) in ops.iter().enumerate() {
        let mut op_use = op.use_registers();
        let mut op_def = op.def_registers();
        op_use.retain(|&reg| reg.is_virtual());
        op_def.retain(|&reg| reg.is_virtual());

        for &vreg in op_use.iter().filter(|reg| reg.is_virtual()) {
            match res.entry(vreg.clone()) {
                hash_map::Entry::Occupied(mut occ) => {
                    occ.get_mut().1.push(idx);
                }
                hash_map::Entry::Vacant(vac) => {
                    vac.insert((vec![], vec![idx]));
                }
            }
        }
        for &vreg in op_def.iter().filter(|reg| reg.is_virtual()) {
            match res.entry(vreg.clone()) {
                hash_map::Entry::Occupied(mut occ) => {
                    occ.get_mut().0.push(idx);
                }
                hash_map::Entry::Vacant(vac) => {
                    vac.insert((vec![idx], vec![]));
                }
            }
        }
    }
    res
}

/// Given an interference graph and a integer k, figure out if the graph k-colorable. Graph
/// coloring is an NP-complete problem, but the algorithm below is a simple stack based
/// approximation that relies on the fact that any node n in the graph that has fewer than k
/// neighbors can always be colored.
///
/// Algorithm:
/// ===============================================================================================
/// 1. Pick any node n such that degree(n) < k and put it on the stack.
/// 2. Remove node n and all its edges from the graph
///    - This may make some new nodes have fewer than k neighbours which is nice.
/// 3. If some vertex n still has k or more neighbors, then the graph may not be k colorable.
///    We still add it to the stack as is, as a potential spill. When popping, if we still
///    can't colour it, then it becomes an actual spill.
///
/// ===============================================================================================
///
pub(crate) fn color_interference_graph(
    interference_graph: &mut InterferenceGraph,
    ops: &[Op],
    live_out: &[BTreeSet<VirtualRegister>],
) -> Result<Vec<NodeIndex>, FxHashSet<VirtualRegister>> {
    let mut stack = Vec::with_capacity(interference_graph.node_count());
    let mut on_stack = FxHashSet::default();
    let mut spills = FxHashSet::default();
    let def_use_points = compute_def_use_points(ops);

    // Nodes with < k-degree before adding to the stack,
    // to have their neighbours processed.
    let mut worklist = vec![];
    // Nodes as yet having >= k-degree.
    let mut pending = FxHashSet::default();

    for node in interference_graph.node_indices() {
        let num_neighbors = interference_graph.neighbors_undirected(node).count();
        if num_neighbors < compiler_constants::NUM_ALLOCATABLE_REGISTERS as usize {
            worklist.push(node);
        } else {
            pending.insert(node);
        }
    }

    // Get outgoing "true" edged neighbors.
    fn get_connected_outgoing_neighbors(
        interference_graph: &InterferenceGraph,
        node_index: NodeIndex,
    ) -> impl Iterator<Item = NodeIndex> + '_ {
        interference_graph
            .edges_directed(node_index, Outgoing)
            .filter_map(|e| interference_graph[e.id()].then_some(e.target()))
    }

    // Get incoming "true" edged neighbors.
    fn get_connected_incoming_neighbors(
        interference_graph: &InterferenceGraph,
        node_index: NodeIndex,
    ) -> impl Iterator<Item = NodeIndex> + '_ {
        interference_graph
            .edges_directed(node_index, Incoming)
            .filter_map(|e| interference_graph[e.id()].then_some(e.source()))
    }

    // Get neighbours (either direction) connected via a "true" edge.
    fn get_connected_neighbours(
        interference_graph: &InterferenceGraph,
        node_index: NodeIndex,
    ) -> impl Iterator<Item = NodeIndex> + '_ {
        get_connected_outgoing_neighbors(interference_graph, node_index).chain(
            get_connected_incoming_neighbors(interference_graph, node_index),
        )
    }

    // Mark edges to/from node satisfying the conditions as deleted.
    fn delete_edges<P: Fn(&VirtualRegister, &VirtualRegister) -> bool>(
        interference_graph: &mut InterferenceGraph,
        node_index: NodeIndex,
        should_delete: P,
    ) {
        let edges: Vec<_> = interference_graph
            .edges_directed(node_index, Outgoing)
            .chain(interference_graph.edges_directed(node_index, Incoming))
            .map(|edge| edge.id())
            .collect();

        for e in edges {
            let (source, target) = interference_graph.edge_endpoints(e).unwrap();
            {
                if should_delete(&interference_graph[source], &interference_graph[target]) {
                    interference_graph[e] = false;
                }
            }
        }
    }

    loop {
        while let Some(node_index) = worklist.pop() {
            // Ensure that we've not already processed this.
            if on_stack.contains(&node_index) {
                continue;
            }

            // This node is colourable.
            stack.push(node_index);
            on_stack.insert(node_index);

            // When spilled, not all edges should be deleted, and the spilling
            // code takes care of deleting the right edges.
            if !spills.contains(&interference_graph[node_index]) {
                // Delete all edges connected to node_index.
                delete_edges(interference_graph, node_index, |_, _| true)
            }

            let candidate_neighbors: Vec<_> = interference_graph
                .neighbors_undirected(node_index)
                .filter(|n| {
                    pending.contains(n)
                        && get_connected_neighbours(interference_graph, *n).count()
                            < compiler_constants::NUM_ALLOCATABLE_REGISTERS as usize
                })
                .collect();
            for candidate_neighbor in &candidate_neighbors {
                pending.remove(candidate_neighbor);
                worklist.push(*candidate_neighbor);
            }
        }

        // At the moment, our spill priority function is just this,
        // i.e., spill the register with more incoming interferences.
        // (roughly indicating how long the interval is).
        if let Some(spill_reg_index) = pending.iter().copied().max_by(|node1, node2| {
            let node1_priority =
                get_connected_incoming_neighbors(interference_graph, *node1).count();
            let node2_priority =
                get_connected_incoming_neighbors(interference_graph, *node2).count();
            match node1_priority.cmp(&node2_priority) {
                Ordering::Equal => {
                    // Equal priorities are broken deterministically and do not alter the spill heuristic.
                    let reg_cmp = interference_graph[*node1].cmp(&interference_graph[*node2]);
                    if reg_cmp == Ordering::Equal {
                        node1.index().cmp(&node2.index())
                    } else {
                        reg_cmp
                    }
                }
                other => other,
            }
        }) {
            let spill_reg = interference_graph[spill_reg_index].clone();
            spills.insert(spill_reg.clone());

            // Update the interference graph that this is spilled.
            // A spill implies a store right after a definition and
            // a load right before a use, forming new tiny live ranges.
            // So we retain only those interferences that correspond to
            // these tiny live ranges and remove the rest.
            let to_retain =
                def_use_points
                    .get(&spill_reg)
                    .map_or(FxHashSet::default(), |(defs, uses)| {
                        let mut retains = FxHashSet::default();
                        for &def in defs {
                            retains
                                .extend(live_out[def].iter().filter(|reg| !spills.contains(*reg)));
                        }
                        for &r#use in uses.iter().filter(|&&r#use| r#use > 0) {
                            retains.extend(
                                live_out[r#use - 1]
                                    .iter()
                                    .filter(|reg| !spills.contains(*reg)),
                            );
                        }
                        retains
                    });

            delete_edges(interference_graph, spill_reg_index, |source, target| {
                !(to_retain.contains(source) || to_retain.contains(target))
            });

            pending.remove(&spill_reg_index);
            worklist.push(spill_reg_index);
        } else {
            break;
        }
    }

    if spills.is_empty() {
        Ok(stack)
    } else {
        Err(spills)
    }
}

/// Assigns an allocatable register to each virtual register used by some instruction in the
/// list `self.ops`. The algorithm used is Chaitin's graph-coloring register allocation
/// algorithm (https://en.wikipedia.org/wiki/Chaitin%27s_algorithm). The individual steps of
/// the algorithm are thoroughly explained in register_allocator.rs.
pub(crate) fn allocate_registers(ops: &[Op]) -> Result<Vec<AllocatedAbstractOp>, CompileError> {
    enum ColouringResult {
        Success {
            updated_ops: Vec<Op>,
            interference_graph: InterferenceGraph,
            colouring_stack: Vec<NodeIndex>,
        },
        SpillsNeeded {
            updated_ops: Vec<Op>,
            spills: FxHashSet<VirtualRegister>,
        },
    }

    fn try_color(ops: &[Op]) -> ColouringResult {
        // Step 1: Liveness Analysis.
        let live_out = liveness_analysis(ops, true);

        // Step 2: Construct the interference graph.
        let (mut interference_graph, mut reg_to_node_ix) =
            create_interference_graph(ops, &live_out);

        // Step 3: Remove redundant MOVE instructions using the interference graph.
        let (updated_ops, live_out) =
            coalesce_registers(ops, live_out, &mut interference_graph, &mut reg_to_node_ix);

        // Step 4: Simplify - i.e. color the interference graph and return a stack that contains
        // each colorable node and its neighbors.
        match color_interference_graph(&mut interference_graph, &updated_ops, &live_out) {
            Ok(colouring_stack) => ColouringResult::Success {
                updated_ops,
                interference_graph,
                colouring_stack,
            },
            Err(spills) => ColouringResult::SpillsNeeded {
                updated_ops,
                spills,
            },
        }
    }

    // We start with the ops we're given.
    let mut updated_ops_ref = ops;
    // A placeholder for updated ops.
    let mut updated_ops;
    // How many times to try spilling before we give up.
    let mut try_count = 0;
    // Try and assign registers. If we fail, spill. Repeat few times.
    let (updated_ops, interference_graph, mut stack) = loop {
        match try_color(updated_ops_ref) {
            ColouringResult::Success {
                updated_ops,
                interference_graph,
                colouring_stack,
            } => {
                break (updated_ops, interference_graph, colouring_stack);
            }
            ColouringResult::SpillsNeeded {
                updated_ops: updated_ops_before_spill,
                spills,
            } => {
                if try_count >= 4 {
                    let comment = updated_ops_before_spill
                        .iter()
                        .find_map(|op| {
                            if let Either::Right(crate::asm_lang::ControlFlowOp::Label(_)) =
                                op.opcode
                            {
                                Some(op.comment.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or("unknown".into());

                    return Err(CompileError::InternalOwned(
                        format!(
                            "The allocator cannot resolve a register mapping for function {comment}. \
                                     Using #[inline(never)] on some functions may help."
                        ),
                        Span::dummy(),
                    ));
                }
                try_count += 1;
                updated_ops = spill(&updated_ops_before_spill, &spills);
                updated_ops_ref = &updated_ops;
            }
        }
    };

    // Step 5: Use the stack to assign a register for each virtual register.
    let pool = assign_registers(&interference_graph, &mut stack)?;
    // Step 6: Update all instructions to use the resulting register pool.
    let mut buf = vec![];
    for op in &updated_ops {
        buf.push(AllocatedAbstractOp {
            opcode: op.allocate_registers(&pool),
            comment: op.comment.clone(),
            owning_span: op.owning_span.clone(),
        })
    }

    Ok(buf)
}

/// Use the stack generated by the coloring algorithm to figure out a register assignment for each
/// virtual register. The idea here is to successively pop the stack while selecting a register to
/// each virtual register. A register r is available to a virtual register v if the intersection of
/// the neighbors of v (available from the stack) and the list of virtual registers already used by
/// r (available in the used_by field) is empty.
///
fn assign_registers(
    interference_graph: &InterferenceGraph,
    stack: &mut Vec<NodeIndex>,
) -> Result<RegisterPool, CompileError> {
    let mut pool = RegisterPool::init();

    while let Some(node) = stack.pop() {
        let reg = interference_graph[node].clone();
        let neighbors: BTreeSet<VirtualRegister> = interference_graph
            .neighbors_undirected(node)
            .map(|neighbor| interference_graph[neighbor].clone())
            .collect();
        if reg.is_virtual() {
            let available =
                pool.registers
                    .iter_mut()
                    .find(|RegisterAllocationStatus { reg: _, used_by }| {
                        neighbors.intersection(used_by).count() == 0
                    });

            if let Some(RegisterAllocationStatus { reg: _, used_by }) = available {
                used_by.insert(reg.clone());
            } else {
                return Err(CompileError::Internal(
                    "The allocator cannot resolve a register mapping for this program. \
                             Using #[inline(never)] on some functions may help.",
                    Span::dummy(),
                ));
            }
        }
    }

    Ok(pool)
}

/// Given a function, its locals info (stack frame usage details)
/// and a set of virtual registers to be spilled, insert the actual spills
/// and return the updated function and the updated stack info.
fn spill(ops: &[Op], spills: &FxHashSet<VirtualRegister>) -> Vec<Op> {
    let mut spilled: Vec<Op> = vec![];

    // Attempt to discover the current stack size and base register.
    let mut cfe_idx_opt = None;
    let mut cfs_idx_opt = None;
    for (op_idx, op) in ops.iter().enumerate() {
        match &op.opcode {
            Either::Left(VirtualOp::CFEI(..)) => {
                assert!(cfe_idx_opt.is_none(), "Found more than one stack extension");
                cfe_idx_opt = Some(op_idx);
            }
            Either::Left(VirtualOp::CFSI(..)) => {
                assert!(cfs_idx_opt.is_none(), "Found more than one stack shrink");
                cfs_idx_opt = Some(op_idx);
            }
            _ => (),
        }
    }

    let cfe_idx = cfe_idx_opt.expect("Function does not have CFEI instruction for locals");

    let Either::Left(VirtualOp::CFEI(
        VirtualRegister::Constant(ConstantRegister::StackPointer),
        virt_imm_24,
    )) = &ops[cfe_idx].opcode
    else {
        panic!("Unexpected opcode");
    };
    let locals_size_bytes = virt_imm_24.value();

    // pad up the locals size in bytes to a word.
    let locals_size_bytes = size_bytes_round_up_to_word_alignment!(locals_size_bytes);

    // Determine the stack slots for each spilled register.
    let spill_offsets_bytes = spill_offsets(spills, locals_size_bytes);

    let spills_size = (8 * spills.len()) as u32;
    let new_locals_byte_size = locals_size_bytes + spills_size;
    if new_locals_byte_size > compiler_constants::TWENTY_FOUR_BITS as u32 {
        panic!("Enormous stack usage for locals.");
    }

    for (op_idx, op) in ops.iter().enumerate() {
        if op_idx == cfe_idx {
            // This is the CFE instruction, use the new stack size.
            spilled.push(Op {
                opcode: Either::Left(VirtualOp::CFEI(
                    VirtualRegister::Constant(ConstantRegister::StackPointer),
                    VirtualImmediate24::new(new_locals_byte_size.into()),
                )),
                comment: op.comment.clone() + &format!(", register spills {spills_size} byte(s)"),
                owning_span: op.owning_span.clone(),
            });
        } else if matches!(cfs_idx_opt, Some(cfs_idx) if cfs_idx == op_idx) {
            // This is the CFS instruction, use the new stack size.
            spilled.push(Op {
                opcode: Either::Left(VirtualOp::CFSI(
                    VirtualRegister::Constant(ConstantRegister::StackPointer),
                    VirtualImmediate24::new(new_locals_byte_size.into()),
                )),
                comment: op.comment.clone() + &format!(", register spills {spills_size} byte(s)"),
                owning_span: op.owning_span.clone(),
            });
        } else {
            // For every other instruction:
            //   If it defines a spilled register, store that register to its stack slot.
            //   If it uses a spilled register, load that register from its stack slot.
            let use_registers = op.use_registers();
            let def_registers = op.def_registers();

            // Calculate the address off a local in a register + imm word offset.
            fn calculate_offset_reg_wordimm(
                inst_list: &mut Vec<Op>,
                offset_bytes: u32,
            ) -> (VirtualRegister, VirtualImmediate12) {
                assert!(offset_bytes.is_multiple_of(8));
                if offset_bytes <= compiler_constants::EIGHTEEN_BITS as u32 {
                    let offset_mov_instr = Op {
                        opcode: Either::Left(VirtualOp::MOVI(
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate18::new(offset_bytes.into()),
                        )),
                        comment: "[spill/refill]: set offset".to_string(),
                        owning_span: None,
                    };
                    inst_list.push(offset_mov_instr);
                    let offset_add_instr = Op {
                        opcode: Either::Left(VirtualOp::ADD(
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualRegister::Constant(ConstantRegister::LocalsBase),
                        )),
                        comment: "[spill/refill]: add offset to stack base".to_string(),
                        owning_span: None,
                    };
                    inst_list.push(offset_add_instr);
                    (
                        VirtualRegister::Constant(ConstantRegister::Scratch),
                        VirtualImmediate12::new(0),
                    )
                } else {
                    assert!(offset_bytes <= compiler_constants::TWENTY_FOUR_BITS as u32);
                    // To have a 24b immediate value, we split it into 12-12 bits
                    // The upper 12 bits are shifted down, then put in a register using
                    // MOVi and then shifted back up via SLLI. Adding back the lower 12 bits
                    // gives us back the original value. We first add the locals_base register
                    // though and then just return the lower 12 bits (but in words) to be used
                    // as an imm value and added in the consumer LW/SW.
                    let offset_upper_12 = offset_bytes >> 12;
                    let offset_lower_12 = offset_bytes & 0b111111111111;
                    assert!((offset_upper_12 << 12) + offset_lower_12 == offset_bytes);
                    let offset_upper_mov_instr = Op {
                        opcode: Either::Left(VirtualOp::MOVI(
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate18::new(offset_upper_12.into()),
                        )),
                        comment: "[spill/refill]: compute offset".to_string(),
                        owning_span: None,
                    };
                    inst_list.push(offset_upper_mov_instr);
                    let offset_upper_shift_instr = Op {
                        opcode: Either::Left(VirtualOp::SLLI(
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualImmediate12::new(12),
                        )),
                        comment: "[spill/refill]: compute offset".to_string(),
                        owning_span: None,
                    };
                    inst_list.push(offset_upper_shift_instr);
                    let offset_add_instr = Op {
                        opcode: Either::Left(VirtualOp::ADD(
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualRegister::Constant(ConstantRegister::Scratch),
                            VirtualRegister::Constant(ConstantRegister::LocalsBase),
                        )),
                        comment: "[spill/refill]: compute offset".to_string(),
                        owning_span: None,
                    };
                    inst_list.push(offset_add_instr);
                    (
                        VirtualRegister::Constant(ConstantRegister::Scratch),
                        // This will be multiplied by 8 by the VM
                        VirtualImmediate12::new((offset_lower_12 / 8).into()),
                    )
                }
            }

            // Take care of any refills on the uses.
            for &spilled_use in use_registers.iter().filter(|r#use| spills.contains(r#use)) {
                // Load the spilled register from its stack slot.
                let offset_bytes = spill_offsets_bytes[spilled_use];
                assert!(offset_bytes.is_multiple_of(8));
                if offset_bytes / 8 <= compiler_constants::TWELVE_BITS as u32 {
                    spilled.push(Op {
                        opcode: Either::Left(VirtualOp::LW(
                            spilled_use.clone(),
                            VirtualRegister::Constant(ConstantRegister::LocalsBase),
                            // This will be multiplied by 8 by the VM
                            VirtualImmediate12::new((offset_bytes / 8).into()),
                        )),
                        comment: "[spill/refill]: refill from spill".to_string(),
                        owning_span: None,
                    });
                } else {
                    let (offset_reg, offset_imm_word) =
                        calculate_offset_reg_wordimm(&mut spilled, offset_bytes);
                    let lw = Op {
                        opcode: Either::Left(VirtualOp::LW(
                            spilled_use.clone(),
                            offset_reg,
                            // This will be multiplied by 8 by the VM
                            offset_imm_word,
                        )),
                        comment: "[spill/refill]: refill from spill".to_string(),
                        owning_span: None,
                    };
                    spilled.push(lw);
                }
            }

            // The op itself.
            spilled.push(op.clone());

            // Take care of spills from the def registers.
            for &spilled_def in def_registers.iter().filter(|def| spills.contains(def)) {
                // Store the def register to its stack slot.
                let offset_bytes = spill_offsets_bytes[spilled_def];
                assert!(offset_bytes.is_multiple_of(8));
                if offset_bytes / 8 <= compiler_constants::TWELVE_BITS as u32 {
                    spilled.push(Op {
                        opcode: Either::Left(VirtualOp::SW(
                            VirtualRegister::Constant(ConstantRegister::LocalsBase),
                            spilled_def.clone(),
                            // This will be multiplied by 8 by the VM
                            VirtualImmediate12::new((offset_bytes / 8).into()),
                        )),
                        comment: "[spill/refill]: spill".to_string(),
                        owning_span: None,
                    });
                } else {
                    let (offset_reg, offset_imm_word) =
                        calculate_offset_reg_wordimm(&mut spilled, offset_bytes);
                    let sw = Op {
                        opcode: Either::Left(VirtualOp::SW(
                            offset_reg,
                            spilled_def.clone(),
                            // This will be multiplied by 8 by the VM
                            offset_imm_word,
                        )),
                        comment: "[spill/refill]: spill".to_string(),
                        owning_span: None,
                    };
                    spilled.push(sw);
                }
            }
        }
    }

    spilled
}

fn spill_offsets(
    spills: &FxHashSet<VirtualRegister>,
    locals_size_bytes: u32,
) -> FxHashMap<VirtualRegister, u32> {
    let mut spill_regs: Vec<_> = spills.iter().collect();
    spill_regs.sort();
    spill_regs
        .into_iter()
        .enumerate()
        .map(|(i, reg)| (reg.clone(), (i * 8) as u32 + locals_size_bytes))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hash::FxHashSet;

    fn make_reg(name: &str) -> VirtualRegister {
        VirtualRegister::Virtual(name.to_owned())
    }

    #[test]
    fn spill_offsets_are_deterministic() {
        let locals_size_bytes = 24u32;

        let mut set_a = FxHashSet::default();
        set_a.insert(make_reg("r1"));
        set_a.insert(make_reg("r2"));
        set_a.insert(make_reg("r3"));

        let mut set_b = FxHashSet::default();
        set_b.insert(make_reg("r3"));
        set_b.insert(make_reg("r1"));
        set_b.insert(make_reg("r2"));

        let offsets_a = spill_offsets(&set_a, locals_size_bytes);
        let offsets_b = spill_offsets(&set_b, locals_size_bytes);

        assert_eq!(offsets_a, offsets_b);

        let mut sorted: Vec<_> = offsets_a.into_iter().collect();
        sorted.sort_by(|(reg_l, _), (reg_r, _)| reg_l.cmp(reg_r));
        assert_eq!(
            sorted,
            vec![
                (make_reg("r1"), locals_size_bytes),
                (make_reg("r2"), locals_size_bytes + 8),
                (make_reg("r3"), locals_size_bytes + 16),
            ]
        );
        assert_eq!(offsets_b.len(), 3);
    }
}
