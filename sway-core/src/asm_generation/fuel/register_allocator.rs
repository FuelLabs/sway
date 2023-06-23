use crate::{
    asm_generation::fuel::compiler_constants,
    asm_lang::{
        allocated_ops::AllocatedRegister, virtual_register::*, ControlFlowOp, Label, Op,
        VirtualImmediate12, VirtualImmediate18, VirtualImmediate24, VirtualOp,
    },
};

use either::Either;
use petgraph::graph::NodeIndex;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::{BTreeSet, HashMap};
use sway_error::error::CompileError;
use sway_types::Span;

use super::register_sequencer::RegisterSequencer;

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
pub(crate) fn liveness_analysis(ops: &[Op]) -> Vec<FxHashSet<VirtualRegister>> {
    // Vectors representing maps that will reprsent the live_in and live_out tables. Each entry
    // corresponds to an instruction in `ops`.
    let mut live_in: Vec<FxHashSet<VirtualRegister>> = vec![FxHashSet::default(); ops.len()];
    let mut live_out: Vec<FxHashSet<VirtualRegister>> = vec![FxHashSet::default(); ops.len()];
    let mut label_to_index: HashMap<Label, usize> = HashMap::new();

    // Keep track of an map between jump labels and op indices. Useful to compute op successors.
    for (idx, op) in ops.iter().enumerate() {
        if let Either::Right(ControlFlowOp::Label(op_label)) = op.opcode {
            label_to_index.insert(op_label, idx);
        }
    }

    let mut modified = true;
    while modified {
        modified = false;
        // Iterate in reverse topological order of the CFG (which is basically the same as the
        // reverse order of `ops`. This makes the outer `while` loop converge faster.
        for (ix, op) in ops.iter().rev().enumerate() {
            let mut local_modified = false;
            let rev_ix = ops.len() - ix - 1;

            // Get use and def vectors without any of the Constant registers
            let mut op_use = op.use_registers();
            let mut op_def = op.def_registers();
            op_use.retain(|&reg| matches!(reg, VirtualRegister::Virtual(_)));
            op_def.retain(|&reg| matches!(reg, VirtualRegister::Virtual(_)));

            // Compute live_out(op) = live_in(s_1) UNION live_in(s_2) UNION ..., where s1, s_2, ...
            // are successors of op
            for s in &op.successors(rev_ix, ops, &label_to_index) {
                for l in live_in[*s].iter() {
                    local_modified |= live_out[rev_ix].insert(l.clone());
                }
            }

            // Compute live_in(op) = use(op) UNION (live_out(op) - def(op))
            // Add use(op)
            for u in op_use {
                local_modified |= live_in[rev_ix].insert(u.clone());
            }
            // Add live_out(op) - def(op)
            for l in live_out[rev_ix].iter() {
                if !op_def.contains(&l) {
                    local_modified |= live_in[rev_ix].insert(l.clone());
                }
            }

            // Did anything change in this iteration?
            modified |= local_modified;
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
    live_out: &[FxHashSet<VirtualRegister>],
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

    for (ix, regs) in live_out.iter().enumerate() {
        match &ops[ix].opcode {
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
                for v in &ops[ix].def_registers() {
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

                        let r1_neighbours =
                            interference_graph.neighbors(*ix1).collect::<FxHashSet<_>>();
                        let r2_neighbours =
                            interference_graph.neighbors(*ix2).collect::<FxHashSet<_>>();

                        // Using either of the two safety conditions below, it's guaranteed
                        // that we aren't turning a k-colourable graph into one that's not,
                        // by doing the coalescing. Ref: "Coalescing" section in Appel's book.
                        let briggs_safety = r1_neighbours
                            .union(&r2_neighbours)
                            .filter(|&&neighbour| {
                                interference_graph.neighbors(neighbour).count()
                                    >= compiler_constants::NUM_ALLOCATABLE_REGISTERS as usize
                            })
                            .count()
                            < compiler_constants::NUM_ALLOCATABLE_REGISTERS as usize;

                        let george_safety = r2_neighbours.iter().all(|&r2_neighbor| {
                            r1_neighbours.contains(&r2_neighbor)
                                || interference_graph.neighbors(r2_neighbor).count()
                                    < compiler_constants::NUM_ALLOCATABLE_REGISTERS as usize
                        });

                        let safe = briggs_safety || george_safety;

                        // If r1 and r2 are connected in the interference graph (i.e. their
                        // respective liveness ranges overalp), preserve the MOVE instruction by
                        // adding it to reduced_ops
                        if interference_graph.contains_edge(*ix1, *ix2) || !safe {
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
                        for neighbor in r2_neighbours {
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
/// 1. Pick any node n such that degree(n) < k and put it on the stack.
/// 2. Remove node n and all its edges from the graph
///    - This may make some new nodes have fewer than k neighbours which is nice.
/// 3. If some vertex n still has k or more neighbors, then the graph may not be k colorable.
///     We still add it to the stack as is, as a potential spill. When popping, if we still
///     can't colour it, then it becomes an actual spill.
/// ===============================================================================================
///
pub(crate) fn color_interference_graph(
    interference_graph: &InterferenceGraph,
) -> Result<Vec<VirtualRegister>, FxHashSet<VirtualRegister>> {
    let mut stack = Vec::with_capacity(interference_graph.node_count());
    let mut on_stack = vec![false; interference_graph.node_count()];
    let mut spills = FxHashSet::default();

    // Nodes with < k-degree or potential spills,
    // before adding to the stack,
    // to have their neighbours processed.
    let mut worklist = vec![];
    // Nodes as yet having >= k-degree or not yet potentially spilled.
    let mut pending = FxHashSet::default();

    for node in interference_graph
        .node_indices()
        .filter(|&idx| interference_graph[idx].is_some())
    {
        let num_neighbors = interference_graph
            .neighbors(node)
            .filter(|&n| interference_graph[n].is_some())
            .count();
        if num_neighbors < compiler_constants::NUM_ALLOCATABLE_REGISTERS as usize {
            worklist.push(node);
        } else {
            pending.insert(node);
        }
    }

    loop {
        while let Some(node_index) = worklist.pop() {
            // Ensure that we've not already processed this.
            if on_stack[node_index.index()] {
                continue;
            }

            // Assert that we aren't dealing with dead nodes.
            assert!(interference_graph[node_index].is_some());
            // This node is colourable.
            stack.push(interference_graph[node_index].clone().unwrap());
            on_stack[node_index.index()] = true;

            // See if any neighbours can be moved to the worklist, from pending.
            let candidate_neighbors: Vec<_> = interference_graph
                .neighbors(node_index)
                .filter(|n| {
                    interference_graph[*n].is_some()
                        && pending.contains(n)
                        && interference_graph
                            .neighbors(*n)
                            .filter(|n| interference_graph[*n].is_some() && pending.contains(n))
                            .count()
                            < compiler_constants::NUM_ALLOCATABLE_REGISTERS as usize
                })
                .collect();
            for candidate_neighbor in &candidate_neighbors {
                pending.remove(candidate_neighbor);
                worklist.push(*candidate_neighbor);
            }
        }

        if let Some(&spill_reg) = pending.iter().next() {
            pending.remove(&spill_reg);
            spills.insert(interference_graph[spill_reg].clone().unwrap());
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

/// Use the stack generated by the coloring algorithm to figure out a register assignment for each
/// virtual register. The idea here is to successively pop the stack while selecting a register to
/// each virtual register. A register r is available to a virtual register v if the intersection of
/// the neighbors of v (available from the stack) and the list of virtual registers already used by
/// r (available in the used_by field) is empty.
///
pub(crate) fn assign_registers(
    interference_graph: &InterferenceGraph,
    reg_to_node_map: &HashMap<VirtualRegister, NodeIndex>,
    stack: &mut Vec<VirtualRegister>,
) -> Result<RegisterPool, CompileError> {
    let mut pool = RegisterPool::init();

    while let Some(reg) = stack.pop() {
        let node = reg_to_node_map[&reg];
        let neighbors: BTreeSet<VirtualRegister> = interference_graph
            .neighbors(node)
            .filter_map(|n| interference_graph[n].clone())
            .collect();
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
pub(crate) fn spill(
    reg_seqr: &mut RegisterSequencer,
    ops: &Vec<Op>,
    spills: &FxHashSet<VirtualRegister>,
) -> Vec<Op> {
    let mut spilled: Vec<Op> = vec![];

    // Attempt to discover the current stack size and base register.
    let mut cfe_idx_opt = None;
    let mut cfs_idx_opt = None;
    for (op_idx, op) in ops.iter().enumerate() {
        match &op.opcode {
            Either::Left(VirtualOp::CFEI(_)) => {
                assert!(cfe_idx_opt.is_none(), "Found more than one stack extension");
                cfe_idx_opt = Some(op_idx);
            }
            Either::Left(VirtualOp::CFSI(_)) => {
                assert!(cfs_idx_opt.is_none(), "Found more than one stack shrink");
                cfs_idx_opt = Some(op_idx);
            }
            _ => (),
        }
    }

    let cfe_idx = cfe_idx_opt.expect("Function does not have CFEI instruction for locals");

    let Either::Left(VirtualOp::CFEI(VirtualImmediate24 { value: locals_size })) = ops[cfe_idx].opcode else {
            panic!("Unexpected opcode");
        };

    // Determine the stack slots for each spilled register.
    let spill_word_offsets: FxHashMap<VirtualRegister, u32> = spills
        .iter()
        .enumerate()
        .map(|(i, reg)| (reg.clone(), i as u32 + locals_size))
        .collect();

    let new_locals_byte_size = locals_size + (8 * spills.len()) as u32;
    if new_locals_byte_size > compiler_constants::TWENTY_FOUR_BITS as u32 {
        panic!("Enormous stack usage for locals.");
    }

    for (op_idx, op) in ops.iter().enumerate() {
        if op_idx == cfe_idx {
            // This is the CFE instruction, use the new stack size.
            spilled.push(Op {
                opcode: Either::Left(VirtualOp::CFEI(VirtualImmediate24 {
                    value: new_locals_byte_size as u32,
                })),
                comment: op.comment.clone(),
                owning_span: op.owning_span.clone(),
            });
        } else if cfs_idx_opt.is_some_and(|cfs_idx| cfs_idx == op_idx) {
            // This is the CFS instruction, use the new stack size.
            spilled.push(Op {
                opcode: Either::Left(VirtualOp::CFSI(VirtualImmediate24 {
                    value: new_locals_byte_size as u32,
                })),
                comment: op.comment.clone(),
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
                reg_seqr: &mut RegisterSequencer,
                inst_list: &mut Vec<Op>,
                offset_bytes: u32,
            ) -> (VirtualRegister, VirtualImmediate12) {
                if offset_bytes <= compiler_constants::EIGHTEEN_BITS as u32 {
                    let offset_mov_reg = reg_seqr.next();
                    let offset_mov_instr = Op {
                        opcode: Either::Left(VirtualOp::MOVI(
                            offset_mov_reg.clone(),
                            VirtualImmediate18 {
                                value: offset_bytes as u32,
                            },
                        )),
                        comment: "Spill/Refill: Set offset".to_string(),
                        owning_span: None,
                    };
                    inst_list.push(offset_mov_instr);
                    let offset_add_reg = reg_seqr.next();
                    let offset_add_instr = Op {
                        opcode: Either::Left(VirtualOp::ADD(
                            offset_add_reg.clone(),
                            offset_mov_reg,
                            VirtualRegister::Constant(ConstantRegister::LocalsBase),
                        )),
                        comment: "Spill/Refill: Add offset to stack base".to_string(),
                        owning_span: None,
                    };
                    inst_list.push(offset_add_instr);
                    (offset_add_reg, VirtualImmediate12 { value: 0 })
                } else {
                    assert!(offset_bytes <= compiler_constants::TWENTY_FOUR_BITS as u32);
                    let offset_upper_12 = offset_bytes >> 12;
                    let offset_lower_12 = offset_bytes & 0b111111111111;
                    assert!((offset_upper_12 << 12) + offset_lower_12 == offset_bytes);
                    let offset_upper_mov_reg = reg_seqr.next();
                    let offset_upper_mov_instr = Op {
                        opcode: Either::Left(VirtualOp::MOVI(
                            offset_upper_mov_reg.clone(),
                            VirtualImmediate18 {
                                value: offset_upper_12 as u32,
                            },
                        )),
                        comment: "Spill/Refill: Offset computation".to_string(),
                        owning_span: None,
                    };
                    inst_list.push(offset_upper_mov_instr);
                    let offset_upper_shift_reg = reg_seqr.next();
                    let offset_upper_shift_instr = Op {
                        opcode: Either::Left(VirtualOp::SLLI(
                            offset_upper_shift_reg.clone(),
                            offset_upper_mov_reg,
                            VirtualImmediate12 { value: 12 },
                        )),
                        comment: "Spill/Refill: Offset computation".to_string(),
                        owning_span: None,
                    };
                    inst_list.push(offset_upper_shift_instr);
                    let offset_add_reg = reg_seqr.next();
                    let offset_add_instr = Op {
                        opcode: Either::Left(VirtualOp::ADD(
                            offset_add_reg.clone(),
                            offset_upper_shift_reg,
                            VirtualRegister::Constant(ConstantRegister::LocalsBase),
                        )),
                        comment: "Spill/Refill: Offset computation".to_string(),
                        owning_span: None,
                    };
                    inst_list.push(offset_add_instr);
                    (
                        offset_add_reg,
                        VirtualImmediate12 {
                            // This will be multiplied by 8 by the VM
                            value: (offset_lower_12 / 8) as u16,
                        },
                    )
                }
            }

            // Take care of any refills on the uses.
            for &spilled_use in use_registers.iter().filter(|r#use| spills.contains(r#use)) {
                // Load the spilled register from its stack slot.
                let offset_words = spill_word_offsets[spilled_use];
                if offset_words <= compiler_constants::TWELVE_BITS as u32 {
                    spilled.push(Op {
                        opcode: Either::Left(VirtualOp::LW(
                            spilled_use.clone(),
                            VirtualRegister::Constant(ConstantRegister::LocalsBase),
                            VirtualImmediate12 {
                                value: offset_words as u16,
                            },
                        )),
                        comment: "Refilling from spill".to_string(),
                        owning_span: None,
                    });
                } else {
                    let (offset_reg, offset_imm_word) =
                        calculate_offset_reg_wordimm(reg_seqr, &mut spilled, offset_words * 8);
                    let lw = Op {
                        opcode: Either::Left(VirtualOp::LW(
                            spilled_use.clone(),
                            offset_reg,
                            // This will be multiplied by 8 by the VM
                            offset_imm_word,
                        )),
                        comment: "Refilling from spill".to_string(),
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
                let offset_words = spill_word_offsets[spilled_def];
                if offset_words <= compiler_constants::TWELVE_BITS as u32 {
                    spilled.push(Op {
                        opcode: Either::Left(VirtualOp::SW(
                            VirtualRegister::Constant(ConstantRegister::LocalsBase),
                            spilled_def.clone(),
                            VirtualImmediate12 {
                                value: offset_words as u16,
                            },
                        )),
                        comment: "Spill".to_string(),
                        owning_span: None,
                    });
                } else {
                    let (offset_reg, offset_imm_word) =
                        calculate_offset_reg_wordimm(reg_seqr, &mut spilled, offset_words * 8);
                    let sw = Op {
                        opcode: Either::Left(VirtualOp::SW(
                            offset_reg,
                            spilled_def.clone(),
                            // This will be multiplied by 8 by the VM
                            offset_imm_word,
                        )),
                        comment: "Spill".to_string(),
                        owning_span: None,
                    };
                    spilled.push(sw);
                }
            }
        }
    }

    spilled
}
