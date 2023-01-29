use crate::{
    asm_generation::fuel::compiler_constants,
    asm_lang::{allocated_ops::AllocatedRegister, virtual_register::*, Op, VirtualOp},
};

use std::collections::{BTreeSet, HashMap};

use either::Either;
use petgraph::graph::{node_index, NodeIndex};

pub type InterferenceGraph =
    petgraph::stable_graph::StableGraph<Option<VirtualRegister>, (), petgraph::Undirected>;

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
///         live_out(op) = live_in(s_1) UNION live_in(s_2) UNION live_in(s_3) UNION ...
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
pub(crate) fn liveness_analysis(ops: &[Op]) -> HashMap<usize, BTreeSet<VirtualRegister>> {
    // Hash maps that will reprsent the live_in and live_out tables. The key of each hash map is
    // simply the index of each instruction in the `ops` vector.
    let mut live_in: HashMap<usize, BTreeSet<VirtualRegister>> =
        HashMap::from_iter((0..ops.len()).into_iter().map(|idx| (idx, BTreeSet::new())));
    let mut live_out: HashMap<usize, BTreeSet<VirtualRegister>> =
        HashMap::from_iter((0..ops.len()).into_iter().map(|idx| (idx, BTreeSet::new())));

    let mut modified = true;
    while modified {
        modified = false;
        // Iterate in reverse topological order of the CFG (which is basically the same as the
        // reverse order of `ops`. This makes the outer `while` loop converge faster.
        for (ix, op) in ops.iter().rev().enumerate() {
            let rev_ix = ops.len() - ix - 1;

            // Get use and def vectors without any of the Constant registers
            let mut op_use = op.use_registers();
            let mut op_def = op.def_registers();
            op_use.retain(|&reg| matches!(reg, VirtualRegister::Virtual(_)));
            op_def.retain(|&reg| matches!(reg, VirtualRegister::Virtual(_)));

            let prev_live_out_op = live_out.get(&rev_ix).expect("ix must exist").clone();
            let prev_live_in_op = live_in.get(&rev_ix).expect("ix must exist").clone();

            // Compute live_out(op) = live_in(s_1) UNION live_in(s_2) UNION ..., where s1, s_2, ...
            // are successors of op
            let live_out_op = live_out.get_mut(&rev_ix).expect("ix must exist");
            for s in &op.successors(rev_ix, ops) {
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
            modified |= (prev_live_in_op != *live_in_op) || (prev_live_out_op != *live_out_op);
        }
    }

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
    ops: &[Op],
    live_out: &HashMap<usize, BTreeSet<VirtualRegister>>,
) -> (InterferenceGraph, HashMap<VirtualRegister, NodeIndex>) {
    let mut interference_graph = InterferenceGraph::with_capacity(0, 0);

    // Figure out a mapping between a given VirtualRegister and its corresponding NodeIndex
    // in the interference graph.
    let mut reg_to_node_map: HashMap<VirtualRegister, NodeIndex> = HashMap::new();

    // Get all virtual registers used by the intermediate assembly and add them to the graph
    ops.iter()
        .fold(BTreeSet::new(), |mut tree, elem| {
            let mut regs = elem.registers();
            regs.retain(|&reg| matches!(reg, VirtualRegister::Virtual(_)));
            tree.extend(regs.into_iter());
            tree
        })
        .iter()
        .for_each(|&reg| {
            reg_to_node_map.insert(reg.clone(), interference_graph.add_node(Some(reg.clone())));
        });

    for (ix, regs) in live_out {
        match &ops[*ix].opcode {
            Either::Left(VirtualOp::MOVE(v, c)) => {
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
                for v in &ops[*ix].def_registers() {
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
/// updated, as well as the immediate values for some or all jump instructions (`ji`, `jnei`, and
/// `jnzi for now).
///
pub(crate) fn coalesce_registers(
    ops: &[Op],
    interference_graph: &mut InterferenceGraph,
    reg_to_node_map: &mut HashMap<VirtualRegister, NodeIndex>,
) -> Vec<Op> {
    // A map from the virtual registers that are removed to the virtual registers that they are
    // replaced with during the coalescing process.
    let mut reg_to_reg_map: HashMap<&VirtualRegister, &VirtualRegister> = HashMap::new();

    // To hold the final *reduced* list of ops
    let mut reduced_ops: Vec<Op> = Vec::with_capacity(ops.len());

    for op in ops {
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

                        // If r1 and r2 are connected in the interference graph (i.e. their
                        // respective liveness ranges overalp), preserve the MOVE instruction by
                        // adding it to reduced_ops
                        if interference_graph.contains_edge(*ix1, *ix2) {
                            reduced_ops.push(op.clone());
                            continue;
                        }

                        // The MOVE instruction can now be safely removed. That is, we simply don't
                        // add it to the reduced_ops vector. Also, we combine the two nodes ix1 and
                        // ix2 into ix1 and then we remove ix2 from the graph. We also have
                        // to do some bookkeeping.
                        //
                        // Note that because the interference graph is of type StableGraph, the
                        // node index corresponding to each virtual register does not change when
                        // some graph nodes are added or removed.

                        // Add all of ix2(r2)'s edges to `ix1(r1)`
                        for neighbor in interference_graph.neighbors(*ix2).collect::<Vec<_>>() {
                            interference_graph.add_edge(neighbor, *ix1, ());
                        }

                        // Remove ix2 by setting its weight to `None`.
                        interference_graph[*ix2] = None;

                        // Update the register maps
                        reg_to_node_map.insert(r2.clone(), *ix1);
                        reg_to_reg_map.insert(r2, r1);
                    }
                    _ => {
                        // Preserve the MOVE instruction if either registers used in the MOVE is
                        // special registers (i.e. *not* a VirtualRegister::Virtual(_))
                        reduced_ops.push(op.clone());
                    }
                }
            }
            _ => {
                // Preserve all other instructions
                reduced_ops.push(op.clone());
            }
        }
    }

    // Create a *final* reg-to-reg map that We keep looking for mappings within reg_to_reg_map
    // until we find a register that doesn't map to any other.
    let mut final_reg_to_reg_map: HashMap<&VirtualRegister, &VirtualRegister> = HashMap::new();
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
/// As we don't implement spilling just yet, I've modified the algorithm above to assume k=infinity
/// and moving the colorability checking until the register assignment phase. The reason for this
/// is that the algorithm above can be too conservative and may bail our early even though a valid
/// assignment is actually available. We can revisit this decision when decide to implement
/// spilling.
///
pub(crate) fn color_interference_graph(
    interference_graph: &mut InterferenceGraph,
) -> Vec<(VirtualRegister, BTreeSet<VirtualRegister>)> {
    let mut stack = Vec::with_capacity(interference_graph.node_count());

    // Raw for loop here is safe because we are not actually removing any nodes from the graph at
    // any point (i.e. we're never calling `remove_node()`). This means that each `index` below
    // correspond to a valid `NodeIndex` in the graph.
    for index in 0..interference_graph.node_count() {
        // Convert to a `NodeIndex`
        let node = node_index(index);

        // Nodes with weight `None` are dead
        if interference_graph[node].is_none() {
            continue;
        }

        // Grab all neighbors with node weight not equal to `None`
        let neighbors = interference_graph
            .neighbors(node)
            .filter_map(|n| interference_graph[n].clone())
            .collect();

        // Build the stack
        stack.push((interference_graph[node].clone().unwrap(), neighbors));

        // Remove `node` by setting its weight to `None`.
        interference_graph[node] = None;
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
            } else {
                // Error out for now if no available register is found
                unimplemented!(
                    "The allocator cannot resolve a register mapping for this program. \
                     This is a temporary artifact of the extremely early stage version \
                     of this language. Try to lower the number of variables you use."
                );
            }
        }
    }

    pool
}
