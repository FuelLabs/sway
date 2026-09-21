//! Reorder abstract functions to reduce the cost of calls (near vs medium vs far,
//! with a loop-nesting boost).

use std::collections::HashMap;

use either::Either;

use crate::{
    asm_generation::fuel::{abstract_instruction_set::AbstractInstructionSet, compiler_constants},
    asm_lang::{ControlFlowOp, JumpType, Label},
};

/// Greedy heuristic that permutes `fns` (except index 0) to lower a weighted
/// call-distance cost. Intended for release builds only — it increases compile time.
pub(crate) fn optimize_fn_order(fns: &mut Vec<AbstractInstructionSet>) {
    const MAX_ITERS: usize = 128;
    const DEPTH: usize = 10;

    if fns.len() < 2 {
        return;
    }

    let mut layout = FnLayout::new(fns);

    'improve: for _ in 0..MAX_ITERS {
        // Index 0 never moves, so the entry is always original fn 0.
        debug_assert_eq!(layout.order[0], 0);

        let mut found_improvement = false;

        let before_weights = layout.call_weights(None);
        let before_weights_sum = before_weights.values().sum::<usize>();

        // Rank functions by the total (hotness-weighted) cost of the call
        // sites they participate in -- as caller or callee, in either
        // direction. The "worst" function to relocate is the one involved
        // in the most expensive calls, not merely the one with the most
        // back calls. Forward calls are included here: a forward medium/far
        // call is also worth converting to forward near, and a function
        // sitting in such a call would never be reached by a back-call-only
        // ranking.
        let mut fn_cost = layout.fn_call_cost(&before_weights);
        let candidates = &mut fn_cost[1..];
        let depth = DEPTH.min(candidates.len());

        'candidates: for k in 0..depth {
            let (_, (candidate_fn, candidate_cost), _) =
                candidates.select_nth_unstable_by(k, |a, b| b.1.cmp(&a.1));
            if *candidate_cost == 0 {
                break 'candidates;
            }

            let candidate_fn = *candidate_fn;
            let candidate_slot = layout.slot_of(candidate_fn);

            // Try candidate in every possible position
            let mut best_slot = candidate_slot;
            let mut best_weights_sum = before_weights_sum;
            for slot_being_tried in 1..layout.len() {
                let candidate_weights =
                    layout.call_weights(Some((candidate_slot, slot_being_tried)));
                let candidate_sum = candidate_weights.values().sum();

                // Only accept a position where no individual call gets worse
                // and at least one improves
                let no_call_worse = candidate_weights.iter().all(|(call_key, &new_weight)| {
                    let old_weight = before_weights.get(call_key).copied().unwrap_or(0);
                    new_weight <= old_weight
                });
                let some_call_better = candidate_weights.iter().any(|(call_key, &new_weight)| {
                    let old_weight = before_weights.get(call_key).copied().unwrap_or(0);
                    new_weight < old_weight
                });
                let acceptable = no_call_worse && some_call_better;
                if acceptable && candidate_sum < best_weights_sum {
                    best_weights_sum = candidate_sum;
                    best_slot = slot_being_tried;
                }
            }

            // If an improvement was found, accept it
            if best_weights_sum < before_weights_sum {
                layout.relocate(candidate_slot, best_slot);

                // Uncomment this to log improvements
                // eprintln!(
                //     "optimize_fn_order iter {iter}: call cost {before_weights_sum} -> {best_count} (target fn {target_fi} had {target_count} back calls)"
                // );

                found_improvement = true;
                break 'candidates;
            }
        }

        if !found_improvement {
            break 'improve;
        }
    }

    layout.apply();
}

struct CallSite {
    caller: usize,
    offset: u64,
    callee: usize,
}

struct FnLayout<'a> {
    fns: &'a mut Vec<AbstractInstructionSet>,
    fn_sizes: Vec<u64>,
    call_sites: Vec<CallSite>,
    /// Keyed by (caller fn_idx, call-site offset within the caller).
    call_loop_depth: HashMap<(usize, u64), usize>,
    order: Vec<usize>,
}

impl<'a> FnLayout<'a> {
    /// Entry label of a function.
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

    fn new(fns: &'a mut Vec<AbstractInstructionSet>) -> Self {
        let n = fns.len();

        let fn_labels = fns.iter().map(Self::label_of).collect::<Vec<_>>();
        // TODO: We need to measure whether modeling known opts in `worst_case_instruction_size`
        // (e.g. treating removable zero-`CFEI`/`CFSI` or fallthrough jumps as size 0)
        // improves these call-cost estimates here.
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

        // use to boost call inside of loops
        let mut call_loop_depth = HashMap::new();
        for (idx, f) in fns.iter().enumerate() {
            // Index each label in this function to its op position.
            let mut label_idx = HashMap::new();
            for (i, op) in f.ops.iter().enumerate() {
                if let Either::Right(ControlFlowOp::Label(l)) = &op.opcode {
                    label_idx.insert(l.0, i);
                }
            }

            // Find loops
            // Header: The single entry point of a loop that dominates all other nodes within it.
            // Latch: A node inside the loop that has a jump to the header.
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

            // Calculate each call site loop depth
            // The more deep a call is, more boost it will gain
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

    fn slot_of(&self, fn_idx: usize) -> usize {
        self.order
            .iter()
            .position(|&f| f == fn_idx)
            .expect("fn_idx present in order")
    }

    /// `(fn_idx, call cost)` for every function in the current layout.
    ///
    /// The cost of a function is the sum of the hotness-weighted weights of
    /// every call site it participates in -- as caller or callee, in either
    /// direction. A call `A -> B` contributes its weight to both `A` and `B`,
    /// because relocating either endpoint can change that call's distance and
    /// thus its realization (near / medium / far). Unlike a raw back-call
    /// count, this accounts for forward calls too.
    fn fn_call_cost(&self, weights: &HashMap<(usize, u64), usize>) -> Vec<(usize, usize)> {
        let n = self.fn_sizes.len();
        let mut counts = (0..n).map(|fi| (fi, 0usize)).collect::<Vec<_>>();
        for site in &self.call_sites {
            let w = weights
                .get(&(site.caller, site.offset))
                .copied()
                .unwrap_or(0);
            counts[site.caller].1 = counts[site.caller].1.saturating_add(w);
            if site.callee != site.caller {
                counts[site.callee].1 = counts[site.callee].1.saturating_add(w);
            }
        }
        counts
    }

    /// `virtual_move` means (original position, new position).
    /// Returns per-call weights keyed by `(caller fn_idx, call-site offset)`.
    fn call_weights(&self, virtual_move: Option<(usize, usize)>) -> HashMap<(usize, u64), usize> {
        const LOOP_BOOST: usize = 4;

        // fn_idx -> offset of the function's start, in instruction units.
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
                        // Compacted index in the post-remove sequence.
                        let c = if slot < to { slot } else { slot - 1 };
                        self.order[if c < from { c } else { c + 1 }]
                    };
                    fn_to_off[f] = offset;
                    offset += self.fn_sizes[f];
                }
            }
        }

        let mut weights = HashMap::new();
        for site in &self.call_sites {
            let func_start = fn_to_off[site.caller];
            let target_off = fn_to_off[site.callee];
            let call_site = func_start + site.offset;

            let jmp_factor = jmp_weight_factor(target_off, call_site);
            let depth = self
                .call_loop_depth
                .get(&(site.caller, site.offset))
                .copied()
                .unwrap_or(0);

            weights.insert(
                (site.caller, site.offset),
                jmp_factor * LOOP_BOOST.pow(depth as u32),
            );
        }
        weights
    }

    fn relocate(&mut self, from: usize, to: usize) {
        let fi = self.order.remove(from);
        self.order.insert(to, fi);
    }

    fn apply(self) {
        let mut tmp: Vec<_> = std::mem::take(self.fns).into_iter().map(Some).collect();
        *self.fns = self
            .order
            .into_iter()
            .map(|i| tmp[i].take().unwrap())
            .collect();
    }
}

fn jmp_weight_factor(target_off: u64, call_site: u64) -> usize {
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
