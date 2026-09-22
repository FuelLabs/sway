//! Function layout pass: reorder abstract functions to cheapen cross-function calls.
//!
//! # Goal
//!
//! Fuel calls encode differently by distance (`compile_call_inner`): a short forward
//! `JAL` is one instruction; longer ranges need two or three. This pass permutes
//! function bodies (except the first) so more call sites land in cheaper call positions.
//!
//! # Algorithm
//!
//! 1. Try several starting orders: the incoming IR order, a caller-first DFS of the
//!    call graph, and a few deterministic random shuffles.
//! 2. From each start, run simulated annealing (random relocates; sometimes accept
//!    worse call cost) and keep the best order found across all starts.
//! 3. Finish with a greedy hill-climb that only accepts relocates that lower total call cost.

use std::collections::{HashMap, HashSet};

use either::Either;

use crate::{
    asm_generation::fuel::{abstract_instruction_set::AbstractInstructionSet, compiler_constants},
    asm_lang::{ControlFlowOp, JumpType, Label},
};

/// Reorder `fns` to reduce the total call cost.
/// Index `0` is never moved.
pub(crate) fn optimize_fn_order(fns: &mut Vec<AbstractInstructionSet>) {
    if fns.len() < 2 {
        return;
    }

    let mut layout = FnLayout::new(fns);
    let n = layout.len();

    // Seed depends only on `n` so the same program always gets the same layout.
    let mut rng =
        XorShift64::new(0xC0FFEE_F11E_u64 ^ (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));

    let mut seeds = Vec::with_capacity(6);
    seeds.push((0..n).collect::<Vec<_>>());
    seeds.push(caller_first_order(n, &layout.call_sites));
    for _ in 0..4 {
        let mut shuffled = (0..n).collect::<Vec<_>>();
        shuffle_tail(&mut shuffled, &mut rng);
        seeds.push(shuffled);
    }

    let mut best_order = seeds[0].clone();
    layout.order = best_order.clone();
    let mut best_call_cost = layout.total_call_cost();

    for seed in seeds {
        layout.order = seed;
        let (order, call_cost) = layout.simulated_annealing(&mut rng);
        if call_cost < best_call_cost {
            best_call_cost = call_cost;
            best_order = order;
        }
    }

    layout.order = best_order;
    layout.hill_climb_call_cost();
    layout.apply();
}

/// One direct call from `caller` to `callee` at instruction-offset `offset`
/// within the caller's body (same units as `Op::worst_case_instruction_size`).
struct CallSite {
    caller: usize,
    offset: u64,
    callee: usize,
}

/// Working state for evaluating and searching over a permutation of `fns`.
struct FnLayout<'a> {
    fns: &'a mut Vec<AbstractInstructionSet>,
    /// Estimated size of each function in instruction units.
    fn_sizes: Vec<u64>,
    call_sites: Vec<CallSite>,
    /// Nesting depth of each call site inside backward-jump loops, keyed by
    /// `(caller fn_idx, call-site offset)`.
    call_loop_depth: HashMap<(usize, u64), usize>,
    /// Current permutation: `order[slot]` is an index into `fns` / `fn_sizes`.
    order: Vec<usize>,
}

impl<'a> FnLayout<'a> {
    /// First `ControlFlowOp::Label` in the function — its entry label.
    fn label_of(f: &AbstractInstructionSet) -> Label {
        f.ops
            .iter()
            .find_map(|op| match &op.opcode {
                Either::Right(ControlFlowOp::Label(l)) => Some(*l),
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "function {:?} has no Label in its ops (len={})",
                    f.function,
                    f.ops.len()
                )
            })
    }

    /// Collect sizes, call sites, and loop depths; start from identity order.
    fn new(fns: &'a mut Vec<AbstractInstructionSet>) -> Self {
        let n = fns.len();

        let fn_labels = fns.iter().map(Self::label_of).collect::<Vec<_>>();
        // Size estimate before register allocation / later opts. Modeling those
        // opts here (e.g. eliding zero-`CFEI`) is left as a possible refinement.
        let fn_sizes = fns
            .iter()
            .map(|f| f.ops.iter().map(|o| o.worst_case_instruction_size()).sum())
            .collect();
        let label_to_fn_idx: HashMap<Label, usize> = fn_labels
            .iter()
            .enumerate()
            .map(|(idx, &lab)| (lab, idx))
            .collect();

        let mut call_sites = Vec::with_capacity(n);
        for (caller, f) in fns.iter().enumerate() {
            let mut site_off: u64 = 0;
            for op in f.ops.iter() {
                if let Either::Right(ControlFlowOp::Jump {
                    to,
                    ty: JumpType::Call,
                }) = &op.opcode
                {
                    if let Some(&callee) = label_to_fn_idx.get(to) {
                        call_sites.push(CallSite {
                            caller,
                            offset: site_off,
                            callee,
                        });
                    }
                }
                site_off += op.worst_case_instruction_size();
            }
        }

        // Approximate loops via backward jumps to an earlier label in the same fn.
        let mut call_loop_depth = HashMap::new();
        for (idx, f) in fns.iter().enumerate() {
            let mut label_idx = HashMap::new();
            for (i, op) in f.ops.iter().enumerate() {
                if let Either::Right(ControlFlowOp::Label(l)) = &op.opcode {
                    label_idx.insert(l.0, i);
                }
            }

            // header = label index, latch = last backward jump to that label.
            let mut loops = HashMap::new();
            for (latch, op) in f.ops.iter().enumerate() {
                if let Either::Right(ControlFlowOp::Jump { to, ty }) = &op.opcode {
                    match ty {
                        JumpType::Unconditional | JumpType::NotZero(_) => {
                            if let Some(&header) = label_idx.get(&to.0) {
                                if header < latch {
                                    let entry = loops.entry(to.0).or_insert((header, latch));
                                    if latch > entry.1 {
                                        entry.1 = latch;
                                    }
                                }
                            }
                        }
                        JumpType::Call => {}
                    }
                }
            }

            if loops.is_empty() {
                continue;
            }

            let loops = loops.values().cloned().collect::<Vec<_>>();

            // Depth = how many [header, latch] ranges contain the call op.
            let mut site_offset: u64 = 0;
            for (i, op) in f.ops.iter().enumerate() {
                if let Either::Right(ControlFlowOp::Jump {
                    ty: JumpType::Call, ..
                }) = &op.opcode
                {
                    let depth = loops
                        .iter()
                        .filter(|(header, latch)| *header <= i && i <= *latch)
                        .count();
                    if depth > 0 {
                        call_loop_depth.insert((idx, site_offset), depth);
                    }
                }
                site_offset += op.worst_case_instruction_size();
            }
        }

        Self {
            fns,
            fn_sizes,
            call_sites,
            call_loop_depth,
            order: (0..n).collect(),
        }
    }

    fn len(&self) -> usize {
        self.order.len()
    }

    /// Sum of all per-site call costs for the current `order`.
    fn total_call_cost(&self) -> usize {
        self.call_costs(None).values().sum()
    }

    /// Simulated annealing: random relocates, Metropolis accept, geometric cooling.
    ///
    /// 1. Random relocates — repeatedly pick a random function (not index 0) and move it to a random slot.
    /// 2. Metropolis accept — Always accept improvements. Randomly accepts worsenings.
    /// 3. Geometric cooling — After each step, decrease temperature.
    ///
    /// Temperature = how willing are we to accept a worse layout?
    ///
    /// Slot `0` is never chosen. Returns the best order seen and its call cost; also
    /// leaves `self.order` set to that best order.
    fn simulated_annealing(&mut self, rng: &mut XorShift64) -> (Vec<usize>, usize) {
        let n = self.len();
        if n < 3 {
            let call_cost = self.total_call_cost();
            return (self.order.clone(), call_cost);
        }

        let mut current_call_cost = self.total_call_cost();
        let mut best_order = self.order.clone();
        let mut best_call_cost = current_call_cost;

        // Higher initial call cost → higher starting temperature.
        let mut temperature = (current_call_cost as f64 / 5.0).max(2.0);
        let cool = 0.995_f64;

        let iters = (2_000usize).max(40 * n).min(8_000);
        for _ in 0..iters {
            let from = rng.gen_index(1, n);
            let to = rng.gen_index(1, n);
            if from == to {
                continue;
            }

            let new_call_cost: usize = self.call_costs(Some((from, to))).values().sum();
            let delta = new_call_cost as i64 - current_call_cost as i64;

            // if we found an improvement, take it.
            // else, just randomly accept it
            let accept = if delta <= 0 {
                true
            } else {
                let p = (-(delta as f64) / temperature).exp();
                rng.next_f64() < p
            };

            if accept {
                self.relocate(from, to);
                current_call_cost = new_call_cost;
                if current_call_cost < best_call_cost {
                    best_call_cost = current_call_cost;
                    best_order = self.order.clone();
                }
            }

            temperature *= cool;
            if temperature < 0.01 {
                break;
            }
        }

        self.order = best_order.clone();
        (best_order, best_call_cost)
    }

    /// Greedy local search: repeatedly relocate a high-call-cost function to the slot
    /// that most reduces total call cost, until no such move exists (or iter cap).
    fn hill_climb_call_cost(&mut self) {
        const MAX_ITERS: usize = 128;
        const DEPTH: usize = 10;

        'improve: for _ in 0..MAX_ITERS {
            debug_assert_eq!(self.order[0], 0);

            let before_call_costs = self.call_costs(None);
            let before_call_cost_sum = before_call_costs.values().sum::<usize>();

            let mut fn_call_costs = self.fn_call_costs(&before_call_costs);
            let candidates = &mut fn_call_costs[1..];
            let depth = DEPTH.min(candidates.len());

            'candidates: for k in 0..depth {
                let (_, (candidate_fn, candidate_call_cost), _) =
                    candidates.select_nth_unstable_by(k, |a, b| b.1.cmp(&a.1));
                if *candidate_call_cost == 0 {
                    break 'candidates;
                }

                let candidate_fn = *candidate_fn;
                let candidate_slot = self.slot_of(candidate_fn);

                let mut best_slot = candidate_slot;
                let mut best_call_cost_sum = before_call_cost_sum;
                for slot_being_tried in 1..self.len() {
                    let candidate_call_cost_sum: usize = self
                        .call_costs(Some((candidate_slot, slot_being_tried)))
                        .values()
                        .sum();
                    if candidate_call_cost_sum < best_call_cost_sum {
                        best_call_cost_sum = candidate_call_cost_sum;
                        best_slot = slot_being_tried;
                    }
                }

                if best_call_cost_sum < before_call_cost_sum {
                    self.relocate(candidate_slot, best_slot);
                    continue 'improve;
                }
            }

            break 'improve;
        }
    }

    fn slot_of(&self, fn_idx: usize) -> usize {
        self.order
            .iter()
            .position(|&f| f == fn_idx)
            .expect("fn_idx present in order")
    }

    /// Per-function call cost: each site's call cost is added to both caller and callee
    /// (so either end is a relocation candidate).
    fn fn_call_costs(&self, call_costs: &HashMap<(usize, u64), usize>) -> Vec<(usize, usize)> {
        let n = self.fn_sizes.len();
        let mut counts = (0..n).map(|fi| (fi, 0usize)).collect::<Vec<_>>();
        for site in &self.call_sites {
            let call_cost = call_costs
                .get(&(site.caller, site.offset))
                .copied()
                .unwrap_or(0);
            counts[site.caller].1 = counts[site.caller].1.saturating_add(call_cost);
            if site.callee != site.caller {
                counts[site.callee].1 = counts[site.callee].1.saturating_add(call_cost);
            }
        }
        counts
    }

    /// Per-call-site call costs for the current layout, or for a tentative relocate.
    ///
    /// `virtual_move` is `(from_slot, to_slot)`: remove the function at `from_slot`
    /// and insert it at `to_slot` without mutating `order`. Keys are
    /// `(caller fn_idx, call-site offset)`.
    ///
    /// Call cost = [`jmp_call_cost`] × `LOOP_BOOST.pow(loop_depth)`.
    fn call_costs(&self, virtual_move: Option<(usize, usize)>) -> HashMap<(usize, u64), usize> {
        const LOOP_BOOST: usize = 4;

        let n = self.fn_sizes.len();
        let mut fn_to_off = vec![0u64; n];
        let mut offset: u64 = 0;
        match virtual_move {
            None => {
                for &fi in &self.order {
                    fn_to_off[fi] = offset;
                    offset += self.fn_sizes[fi];
                }
            }
            Some((from, to)) => {
                let fi = self.order[from];
                for slot in 0..n {
                    let f = if slot == to {
                        fi
                    } else {
                        // Index into the sequence after removing `from`.
                        let c = if slot < to { slot } else { slot - 1 };
                        self.order[if c < from { c } else { c + 1 }]
                    };
                    fn_to_off[f] = offset;
                    offset += self.fn_sizes[f];
                }
            }
        }

        let mut call_costs = HashMap::new();
        for site in &self.call_sites {
            let func_start = fn_to_off[site.caller];
            let target_off = fn_to_off[site.callee];
            let call_site = func_start + site.offset;

            let distance_call_cost = jmp_call_cost(target_off, call_site);
            let depth = self
                .call_loop_depth
                .get(&(site.caller, site.offset))
                .copied()
                .unwrap_or(0);

            call_costs.insert(
                (site.caller, site.offset),
                distance_call_cost * LOOP_BOOST.pow(depth as u32),
            );
        }
        call_costs
    }

    /// Move the function at slot `from` to slot `to`.
    fn relocate(&mut self, from: usize, to: usize) {
        let fi = self.order.remove(from);
        self.order.insert(to, fi);
    }

    /// Replace `fns` with the permutation in `order`.
    fn apply(self) {
        let mut tmp: Vec<_> = std::mem::take(self.fns).into_iter().map(Some).collect();
        *self.fns = self
            .order
            .into_iter()
            .map(|i| tmp[i].take().unwrap())
            .collect();
    }
}

/// Caller-first preorder DFS of the static call graph.
///
/// Each function is emitted before its callees, which favors forward (callee after
/// caller) layout. Starts at function `0`, then visits any remaining roots in
/// increasing index order (other entries or unreachable helpers).
fn caller_first_order(n: usize, call_sites: &[CallSite]) -> Vec<usize> {
    let mut callees: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut seen_edge: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for site in call_sites {
        if site.callee != site.caller && seen_edge[site.caller].insert(site.callee) {
            callees[site.caller].push(site.callee);
        }
    }

    let mut order = Vec::with_capacity(n);
    let mut visited = vec![false; n];

    fn dfs(fn_idx: usize, callees: &[Vec<usize>], visited: &mut [bool], order: &mut Vec<usize>) {
        if visited[fn_idx] {
            return;
        }
        visited[fn_idx] = true;
        order.push(fn_idx);
        for &c in &callees[fn_idx] {
            dfs(c, callees, visited, order);
        }
    }

    dfs(0, &callees, &mut visited, &mut order);
    for i in 1..n {
        if !visited[i] {
            dfs(i, &callees, &mut visited, &mut order);
        }
    }

    debug_assert_eq!(order.len(), n);
    debug_assert_eq!(order[0], 0);
    order
}

/// Fisher–Yates shuffle of `order[1..]` (index `0` unchanged).
fn shuffle_tail(order: &mut [usize], rng: &mut XorShift64) {
    let n = order.len();
    if n < 3 {
        return;
    }
    for i in (2..n).rev() {
        let j = rng.gen_index(1, i + 1);
        order.swap(i, j);
    }
}

/// Deterministic xorshift64 PRNG (no external dependency).
struct XorShift64(u64);

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self(seed | 1) // avoid the all-zero state
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Uniform index in `[lo, hi)`.
    fn gen_index(&mut self, lo: usize, hi: usize) -> usize {
        debug_assert!(hi > lo);
        lo + (self.next_u64() as usize % (hi - lo))
    }

    fn next_f64(&mut self) -> f64 {
        const MASK: u64 = (1 << 53) - 1;
        (self.next_u64() & MASK) as f64 / ((1u64 << 53) as f64)
    }
}

/// Map call-site → callee distance to a discrete call cost matching `compile_call_inner`.
///
/// | Range                         | Typical lowering        | Cost   |
/// |-------------------------------|-------------------------|-------:|
/// | forward ≤ Imm12               | `JAL $pc iN`            |      1 |
/// | backward, byte Δ ≤ Imm12      | `SUBI` + `JAL`          |      2 |
/// | within Imm18 (MOVI path)      | `MOVI` + ALU + `JAL`    |      3 |
/// | beyond Imm18 (data section)   | load + ALU + `JAL`      |      4 |
///
/// Far costs more than medium so the search still prefers shortening calls that
/// would otherwise spill into the data section, even though both forms use three
/// instructions.
fn jmp_call_cost(target_off: u64, call_site: u64) -> usize {
    const FWD_NEAR: usize = 1;
    const BACK_NEAR: usize = 2;
    const MEDIUM: usize = 3;
    const FAR: usize = 4;

    if target_off >= call_site {
        let delta = target_off - call_site;
        if delta <= compiler_constants::TWELVE_BITS {
            FWD_NEAR
        } else if delta.saturating_sub(1).saturating_mul(4) <= compiler_constants::EIGHTEEN_BITS {
            MEDIUM
        } else {
            FAR
        }
    } else {
        let delta = call_site - target_off;
        if delta.saturating_mul(4) <= compiler_constants::TWELVE_BITS {
            BACK_NEAR
        } else if delta.saturating_add(1).saturating_mul(4) <= compiler_constants::EIGHTEEN_BITS {
            MEDIUM
        } else {
            FAR
        }
    }
}
