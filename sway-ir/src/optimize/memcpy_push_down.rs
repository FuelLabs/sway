//! Branch-aware `memcpy` push-down (lazy copy materialization).
//!
//! A full-object `mem_copy_val dst, src` is turned into a *logical* equivalence
//! `dst == src` and the actual copy is emitted only on paths that need `dst` to
//! become an independent buffer:
//!
//! * Reads of `dst` are delegated to `src` while the copy is still pending.
//! * A partial write to `dst`, any write to `src`, or an opaque use of either
//!   pointer forces the copy to be materialized immediately beforehand.
//! * A full overwrite of `dst` makes the copy dead on that path.
//! * When different states meet at a CFG join, the pending edges are
//!   materialized so that the joined state is uniform.
//!
//! The analysis is deliberately conservative: it only handles copies whose
//! source and destination are single symbols with complete pointer
//! provenance, whose pointers are never captured (stored, returned, or
//! converted to integers), and it bails out if the pending region would
//! cross a CFG cycle (loop), where a sunk copy could otherwise be
//! re-executed. Passing a pointer to a mutable callee is *not* a capture:
//! like `memcpyopt`, this pass assumes callees do not retain argument
//! pointers beyond the call, and materializes the copy before such calls.
//!
//! # Join materialization (soundness)
//!
//! Forcing only the **immediate** Pending predecessors of a live reconcile
//! join is enough for correctness. The copy is inserted on the incoming edge
//! (before that predecessor's terminator). Ancestors above that edge can stay
//! Pending. A deeper backward walk is not required for join soundness.
//!
//! Example (`if` with no `else`, then a shared read after the join) —
//! [`write_inside_if_no_else_read_after.ir`](../../tests/memcpy_push_down/write_inside_if_no_else_read_after.ir):
//!
//! ```text
//! entry ──Pending──► write arm ──Materialized──► join (uses dst)
//!   └──Pending──► fallthrough ──Pending──────────┘
//! ```
//!
//! Join in-state becomes Materialized (uniform private buffer). Because a
//! predecessor is still Pending, sunk analysis marks that edge forced (copy
//! before its `br`). The write arm would materialize before its store. Without
//! covering the fallthrough edge, the join would see a private buffer on one
//! path and an uncopied `dst` on the other. Obligation hoisting (below) then
//! collapses those two sunk sites back to the entry candidate when every path
//! from their nearest common dominator still hits an obligation.
//!
//! Related join-force fixtures:
//! [`sound_join_after_src_mutation.ir`](../../tests/memcpy_push_down/sound_join_after_src_mutation.ir),
//! [`sound_join_pending_nocopy_live.ir`](../../tests/memcpy_push_down/sound_join_pending_nocopy_live.ir).
//!
//! # Obligations and hoisting (profitability)
//!
//! Sunk dataflow first records the set `O` of *obligations* — where a
//! materializing copy would be inserted at the latest safe points:
//!
//! - before a forcing instruction (`Plan::inserts`);
//! - before a forced predecessor terminator (`Plan::edge_inserts`).
//!
//! If `O` is empty, the win is full elimination / delegate-only (delete the
//! candidate and only rewrite reads). There is nothing to hoist.
//!
//! Otherwise let `D` be the nearest common dominator of every point in `O`
//! (still after the candidate becomes Pending). Sibling obligations (e.g. both
//! arms of a branch) share a `D` above the branch; if one obligation already
//! dominated another on the same path, the first materialization would have
//! made that path Materialized, so the later site should not appear in a
//! well-built `O` (enforced by `assert_well_built_obligations`).
//!
//! **Do not always hoist to `D`.** When considering a hoist to `D`, check
//! whether any path from `D` can still avoid the copy (return/revert without
//! hitting `O`). If so, keep the sunk sites — that copy-free path is the
//! profitability win of push-down. Hoist only when every CFG path from `D` to
//! a function exit visits at least one point in `O`.
//!
//! If anticipation fails at `D = ncd(O)`, do not give up on the whole set:
//! partition `O` by dominator-tree children of `D` and retry each subset. A
//! subset `O'` gets its own `D' = ncd(O')`; if `D'` is anticipated, hoist that
//! cluster. Repeat recursively. A side exit from the global `ncd` thus blocks
//! only the too-high merge, not profitable merges in regions below —
//! [`limit_hoist_side_exit_keeps_sunk.ir`](../../tests/memcpy_push_down/limit_hoist_side_exit_keeps_sunk.ir)
//! (singleton arms: sunk already optimal),
//! [`opt_hoist_two_clusters_side_exit.ir`](../../tests/memcpy_push_down/opt_hoist_two_clusters_side_exit.ir)
//! (two diamonds: one copy per region).
//!
//! When hoist applies and `D` is exactly where the candidate already sits
//! (typical if/no-else/join), the pass leaves the entry copy alone rather than
//! sinking to multiple arm sites.
//!
//! Examples (open the IR to see the CFG):
//!
//! * if-write / else-fallthrough / join uses `dst` — both paths hit `O`,
//!   keep one copy before the `if`:
//!   [`write_inside_if_no_else_read_after.ir`](../../tests/memcpy_push_down/write_inside_if_no_else_read_after.ir).
//! * both arms write, then join uses `dst` — same shape:
//!   [`sound_write_both_arms_read_after.ir`](../../tests/memcpy_push_down/sound_write_both_arms_read_after.ir).
//! * if-write / else-revert — else never hits `O`; do not hoist to entry:
//!   [`write_inside_if_else_reverts.ir`](../../tests/memcpy_push_down/write_inside_if_else_reverts.ir).
//! * if-write / else only reads / join does not use `dst` — only the write
//!   arm is in `O`; do not hoist (else stays copy-free):
//!   [`write_inside_if_read_inside_else_join.ir`](../../tests/memcpy_push_down/write_inside_if_read_inside_else_join.ir).
//! * both arms only read, no force — `O` empty; keep delete-entry win:
//!   [`opt_diamond_both_arms_read.ir`](../../tests/memcpy_push_down/opt_diamond_both_arms_read.ir).
//! * obligated join arms + side exit from `ncd(O)` — refuse full hoist; each
//!   arm is a singleton subset so sunk sites stay:
//!   [`limit_hoist_side_exit_keeps_sunk.ir`](../../tests/memcpy_push_down/limit_hoist_side_exit_keeps_sunk.ir).
//! * two independent diamonds + side exit — full `ncd` fails anticipation;
//!   each diamond coalesces on its own:
//!   [`opt_hoist_two_clusters_side_exit.ir`](../../tests/memcpy_push_down/opt_hoist_two_clusters_side_exit.ir).
//! * three regions + side exit:
//!   [`opt_hoist_three_clusters_side_exit.ir`](../../tests/memcpy_push_down/opt_hoist_three_clusters_side_exit.ir).
//! * nested if/no-else under a hot arm + cold side exit — cluster to `hot`:
//!   [`opt_hoist_nested_cluster_under_side_exit.ir`](../../tests/memcpy_push_down/opt_hoist_nested_cluster_under_side_exit.ir).
//!
//! Obligations are only demand locations. After a hoist to a site other than
//! the candidate, rewrites are recomputed so paths that become Materialized
//! earlier do not keep Pending-style delegation to `src`.
//!
//! A forced predecessor with multiple successors is still special: inserting
//! before its terminator runs on every outgoing edge. That is correct when
//! every successor path is obligated (the anticipation check above); otherwise
//! this pass refuses rather than splitting critical edges —
//! [`limit_critical_edge_live_join.ir`](../../tests/memcpy_push_down/limit_critical_edge_live_join.ir)
//! (same Sway as `write_inside_if_no_else_read_after`, but the fallthrough edge
//! is not split, so the pass bails).

use rustc_hash::{FxHashMap, FxHashSet};
use sway_types::FxIndexSet;

use crate::{
    compute_dom_tree_from_po, compute_post_order, get_loaded_ptr_values, get_memory_offsets,
    get_referred_symbol, get_referred_symbols, get_stored_ptr_values, AnalysisResults, Block,
    Context, DomTree, Function, InstOp, Instruction, InstructionInserter, IrError, LocalVar, Pass,
    PassMutability, ReferredSymbols, ScopedPass, Symbol, Type, Value, ValueDatum,
};

pub const MEMCPY_PUSH_DOWN_NAME: &str = "memcpy-push-down";

pub fn create_memcpy_push_down_pass() -> Pass {
    Pass {
        name: MEMCPY_PUSH_DOWN_NAME,
        descr: "Push mem_copy_val down the CFG, materializing only where needed",
        deps: vec![],
        runner: ScopedPass::FunctionPass(PassMutability::Transform(memcpy_push_down)),
    }
}

/// Where the source of a candidate copy can be obtained in any block.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SrcPtr {
    /// A local variable; `get_local` can be re-created anywhere.
    Local(LocalVar),
    /// An entry-block argument value; it dominates every reachable block.
    Arg(Value),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum State {
    /// The copy does not execute on this path.
    NoCopy,
    /// The copy logically happened but was not emitted yet.
    Pending,
    /// The copy was emitted (or `dst` was fully overwritten).
    Materialized,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AccessKind {
    Read,
    /// A write that only covers part of the symbol.
    PartialWrite,
    /// A write covering the whole symbol.
    FullWrite,
    /// A call/asm use that may read and write through the pointer.
    MayWrite,
}

#[derive(Clone, Copy, Debug)]
struct Access {
    sym: Symbol,
    kind: AccessKind,
    /// The pointer operand through which the symbol is accessed.
    ptr: Value,
}

struct Candidate {
    inst: Value,
    block: Block,
    dst_local: LocalVar,
    dst_sym: Symbol,
    src: SrcPtr,
    src_sym: Symbol,
}

/// Does `ptr` refer (completely) to exactly the symbol `sym`?
fn refers_to(context: &Context, ptr: Value, sym: Symbol) -> bool {
    get_referred_symbol(context, ptr) == Some(sym)
}

/// Collect all accesses of `dst_sym`/`src_sym` performed by `inst`.
///
/// Returns `None` if the instruction interacts with either symbol in a way we
/// cannot model precisely (pointer capture, unknown provenance, ...), in
/// which case the whole candidate must be abandoned.
fn classify_inst(
    context: &Context,
    inst: Value,
    dst_sym: Symbol,
    src_sym: Symbol,
) -> Option<Vec<Access>> {
    let instruction = inst.get_instruction(context)?;
    let mut accesses = Vec::new();

    let sym_size = |sym: Symbol| -> u64 {
        sym.get_type(context)
            .get_pointee_type(context)
            .map(|ty| ty.size(context).in_bytes())
            .unwrap_or(0)
    };
    let push_accesses = |ptr_values: &[Value], is_write: bool, accesses: &mut Vec<Access>| {
        for ptr in ptr_values {
            for sym in [dst_sym, src_sym] {
                if refers_to(context, *ptr, sym) {
                    let offset = get_memory_offsets(context, *ptr)
                        .and_then(|m| m.get(&sym).copied())
                        .unwrap_or(u64::MAX);
                    let size = ptr
                        .get_type(context)
                        .and_then(|ty| ty.get_pointee_type(context))
                        .map(|ty| ty.size(context).in_bytes())
                        .unwrap_or(0);
                    let kind = if !is_write {
                        AccessKind::Read
                    } else if offset == 0 && size >= sym_size(sym) && sym_size(sym) > 0 {
                        AccessKind::FullWrite
                    } else {
                        AccessKind::PartialWrite
                    };
                    accesses.push(Access {
                        sym,
                        kind,
                        ptr: *ptr,
                    });
                } else if matches!(
                    get_referred_symbols(context, *ptr),
                    ReferredSymbols::Incomplete(_)
                ) && ptr.get_type(context).is_some_and(|ty| ty.is_ptr(context))
                {
                    // A pointer we cannot resolve might alias either symbol,
                    // but only if the instruction actually dereferences it.
                    // Both caller lists only contain dereferenced pointers,
                    // so be safe and give up.
                    return false;
                }
            }
        }
        true
    };

    match &instruction.op {
        InstOp::Call(callee, args) => {
            for (idx, arg) in args.iter().enumerate() {
                for sym in [dst_sym, src_sym] {
                    if refers_to(context, *arg, sym) {
                        let kind = if callee.is_arg_immutable(context, idx) {
                            AccessKind::Read
                        } else {
                            AccessKind::MayWrite
                        };
                        accesses.push(Access {
                            sym,
                            kind,
                            ptr: *arg,
                        });
                    }
                }
            }
        }
        InstOp::AsmBlock(_, args) => {
            for arg in args.iter().filter_map(|a| a.initializer) {
                for sym in [dst_sym, src_sym] {
                    if refers_to(context, arg, sym) {
                        accesses.push(Access {
                            sym,
                            kind: AccessKind::MayWrite,
                            ptr: arg,
                        });
                    }
                }
            }
        }
        InstOp::ContractCall { params, .. } => {
            // Calldata is serialized opaquely; treat any involvement as fatal.
            for sym in [dst_sym, src_sym] {
                if refers_to(context, *params, sym) {
                    return None;
                }
            }
        }
        InstOp::Ret(val, _) | InstOp::PtrToInt(val, _) => {
            // Returning or materializing the address makes the pointer
            // identity observable.
            for sym in [dst_sym, src_sym] {
                if refers_to(context, *val, sym) {
                    return None;
                }
            }
        }
        InstOp::Store { stored_val, .. } => {
            // Storing the pointer itself captures it.
            for sym in [dst_sym, src_sym] {
                if refers_to(context, *stored_val, sym) {
                    return None;
                }
            }
            let stored = get_stored_ptr_values(context, inst);
            if !push_accesses(&stored, true, &mut accesses) {
                return None;
            }
        }
        InstOp::InitAggr(init) => {
            for init_val in &init.initializers {
                for sym in [dst_sym, src_sym] {
                    if refers_to(context, *init_val, sym) {
                        return None;
                    }
                }
            }
            let stored = get_stored_ptr_values(context, inst);
            if !push_accesses(&stored, true, &mut accesses) {
                return None;
            }
        }
        _ => {
            let loaded = get_loaded_ptr_values(context, inst);
            if !push_accesses(&loaded, false, &mut accesses) {
                return None;
            }
            let stored = get_stored_ptr_values(context, inst);
            if !push_accesses(&stored, true, &mut accesses) {
                return None;
            }
            // `mem_copy_bytes` carries an explicit byte length; adjust the
            // write classification for it.
            if let InstOp::MemCopyBytes {
                dst_val_ptr,
                byte_len,
                ..
            } = &instruction.op
            {
                for access in accesses.iter_mut() {
                    if access.ptr == *dst_val_ptr && access.kind != AccessKind::Read {
                        access.kind =
                            if *byte_len >= sym_size(access.sym) && sym_size(access.sym) > 0 {
                                AccessKind::FullWrite
                            } else {
                                AccessKind::PartialWrite
                            };
                    }
                }
            }
        }
    }

    // Any other operand that is one of our pointers but was not classified
    // above (e.g. pointer comparisons) makes the candidate unsafe, unless it
    // is just address computation whose uses are classified at the consumer.
    for operand in instruction.op.get_operands() {
        if !operand
            .get_type(context)
            .is_some_and(|ty| ty.is_ptr(context))
        {
            continue;
        }
        let involved = [dst_sym, src_sym]
            .iter()
            .any(|sym| refers_to(context, operand, *sym));
        if !involved {
            continue;
        }
        let classified = accesses.iter().any(|a| a.ptr == operand);
        let address_only = matches!(
            instruction.op,
            InstOp::GetElemPtr { .. } | InstOp::CastPtr(..)
        );
        if !classified && !address_only {
            return None;
        }
    }

    Some(accesses)
}

/// The chain of address computations leading to `ptr`, rooted at the
/// candidate's destination local.
enum PtrChain {
    Base,
    Gep(Box<PtrChain>, Type, Vec<Value>),
    Cast(Box<PtrChain>, Type),
}

fn analyze_ptr_chain(context: &Context, ptr: Value, dst_local: LocalVar) -> Option<PtrChain> {
    match &context.values[ptr.0].value {
        ValueDatum::Instruction(Instruction {
            op: InstOp::GetLocal(local),
            ..
        }) if *local == dst_local => Some(PtrChain::Base),
        ValueDatum::Instruction(Instruction {
            op:
                InstOp::GetElemPtr {
                    base,
                    elem_ptr_ty,
                    indices,
                },
            ..
        }) => Some(PtrChain::Gep(
            Box::new(analyze_ptr_chain(context, *base, dst_local)?),
            elem_ptr_ty.get_pointee_type(context)?,
            indices.clone(),
        )),
        ValueDatum::Instruction(Instruction {
            op: InstOp::CastPtr(base, ty),
            ..
        }) => Some(PtrChain::Cast(
            Box::new(analyze_ptr_chain(context, *base, dst_local)?),
            *ty,
        )),
        _ => None,
    }
}

/// True when `candidate.inst` already materializes the copy immediately
/// before `force`: only constants / a `get_local` of the destination lie
/// between them in the same block. Stops the pass from re-inserting a copy
/// that a previous rewrite already placed at its final forcing site.
fn candidate_already_at_force(context: &Context, candidate: &Candidate, force: Value) -> bool {
    let mut after_candidate = false;
    for inst in candidate.block.instruction_iter(context) {
        if inst == candidate.inst {
            after_candidate = true;
            continue;
        }
        if !after_candidate {
            continue;
        }
        if inst == force {
            return true;
        }
        if inst.is_constant(context) {
            continue;
        }
        if let Some(Instruction {
            op: InstOp::GetLocal(local),
            ..
        }) = inst.get_instruction(context)
        {
            if *local == candidate.dst_local {
                continue;
            }
        }
        return false;
    }
    false
}

/// Blocks that can reach themselves in the CFG (members of a cycle).
///
/// Note: sink-pruning is *not* used here. That over-approximates by also
/// marking the stem into a loop (e.g. `entry` before a `while`) as cyclic,
/// which would disable every candidate that merely precedes a loop.
fn cycle_blocks(context: &Context, function: Function) -> FxHashSet<Block> {
    let blocks: Vec<Block> = function.block_iter(context).collect();
    let mut in_cycle = FxHashSet::default();
    for &start in &blocks {
        let mut stack = vec![start];
        let mut visited = FxHashSet::default();
        let mut found = false;
        while let Some(block) = stack.pop() {
            for succ in block.successors(context) {
                if succ.block == start {
                    found = true;
                    break;
                }
                if visited.insert(succ.block) {
                    stack.push(succ.block);
                }
            }
            if found {
                break;
            }
        }
        if found {
            in_cycle.insert(start);
        }
    }
    in_cycle
}

fn find_candidates(context: &Context, function: Function) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for (block, inst) in function.instruction_iter(context) {
        let InstOp::MemCopyVal {
            dst_val_ptr,
            src_val_ptr,
        } = inst.get_instruction(context).unwrap().op
        else {
            continue;
        };
        let (dst_val_ptr, src_val_ptr) = (dst_val_ptr, src_val_ptr);
        // Destination must be a whole local variable.
        let dst_local = match &context.values[dst_val_ptr.0].value {
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetLocal(local),
                ..
            }) => *local,
            _ => continue,
        };
        let dst_sym = Symbol::Local(dst_local);
        // Source must be re-materializable in any block: another local, or an
        // entry-block argument.
        let (src, src_sym) = match &context.values[src_val_ptr.0].value {
            ValueDatum::Instruction(Instruction {
                op: InstOp::GetLocal(local),
                ..
            }) => (SrcPtr::Local(*local), Symbol::Local(*local)),
            ValueDatum::Argument(arg) if arg.block.is_entry(context) => {
                (SrcPtr::Arg(src_val_ptr), Symbol::Arg(*arg))
            }
            _ => continue,
        };
        if dst_sym == src_sym || dst_val_ptr.get_type(context) != src_val_ptr.get_type(context) {
            continue;
        }
        candidates.push(Candidate {
            inst,
            block,
            dst_local,
            dst_sym,
            src,
            src_sym,
        });
    }
    candidates
}

struct Plan {
    /// (instruction, old pointer) reads to re-point at the source.
    rewrites: Vec<(Value, Value)>,
    /// Instructions before which a materializing copy must be inserted.
    inserts: Vec<Value>,
    /// Predecessor blocks at whose end a copy must be inserted.
    edge_inserts: Vec<Block>,
    delegated_reads: usize,
    off_block_materializations: usize,
    /// The candidate already sits at a forcing site; keep it instead of
    /// deleting and re-inserting.
    keep_candidate: bool,
}

/// A place where sunk analysis requires a materializing `mem_copy_val`.
#[derive(Clone, Copy, Debug)]
enum Obligation {
    /// Mid-block force: copy immediately before this instruction.
    BeforeInst(Value),
    /// Live reconcile edge: copy immediately before this block's terminator.
    BeforeTerminator(Block),
}

enum Coalesce {
    /// Leave the candidate where it is; do not apply the sunk plan.
    KeepOriginal,
    /// Replace the absorbed sunk obligations with these insert-before points.
    /// Obligations not listed stay at their sunk sites.
    Places {
        absorbed: Vec<Obligation>,
        before: Vec<Value>,
    },
}

struct ClusterCoalesce {
    obligations: Vec<Obligation>,
    placement: Value,
    keep_candidate: bool,
}

fn obligation_block(context: &Context, ob: Obligation) -> Block {
    match ob {
        Obligation::BeforeInst(inst) => inst.get_instruction(context).unwrap().parent,
        Obligation::BeforeTerminator(block) => block,
    }
}

fn obligation_insert_point(context: &Context, ob: Obligation) -> Value {
    match ob {
        Obligation::BeforeInst(inst) => inst,
        Obligation::BeforeTerminator(block) => block
            .instruction_iter(context)
            .next_back()
            .expect("block must have a terminator"),
    }
}

/// Sunk `O` must not contain two sites where one dominates the other: the
/// earlier materialization would already have made that path Materialized.
fn assert_well_built_obligations(
    context: &Context,
    dom_tree: &DomTree,
    obligations: &[Obligation],
) {
    for (i, &a) in obligations.iter().enumerate() {
        let pa = obligation_insert_point(context, a);
        for &b in &obligations[i + 1..] {
            let pb = obligation_insert_point(context, b);
            assert!(
                pa != pb
                    && !dom_tree.dominates_instr(context, pa, pb)
                    && !dom_tree.dominates_instr(context, pb, pa),
                "well-built obligation set must not contain comparable sites \
                 ({a:?} dominates or equals {b:?}, or vice versa)"
            );
        }
    }
}

fn nearest_common_dominator(
    dom_tree: &DomTree,
    blocks: &[Block],
    all_blocks: &[Block],
    entry: Block,
) -> Option<Block> {
    if blocks.is_empty() {
        return None;
    }
    let mut best = entry;
    let mut found = false;
    for &cand in all_blocks {
        if blocks.iter().all(|&b| dom_tree.dominates(cand, b)) {
            if !found || dom_tree.dominates(best, cand) {
                best = cand;
                found = true;
            }
        }
    }
    found.then_some(best)
}

/// True when every CFG path from `start` to a function exit visits at least
/// one block in `obligation_blocks`.
fn is_anticipated(
    context: &Context,
    start: Block,
    obligation_blocks: &FxHashSet<Block>,
) -> bool {
    fn check(
        context: &Context,
        block: Block,
        hit: bool,
        obligation_blocks: &FxHashSet<Block>,
        memo: &mut FxHashMap<(Block, bool), bool>,
        visiting: &mut FxHashSet<(Block, bool)>,
    ) -> bool {
        let hit = hit || obligation_blocks.contains(&block);
        let key = (block, hit);
        if let Some(&ans) = memo.get(&key) {
            return ans;
        }
        if !visiting.insert(key) {
            // Cycle: do not reject on backedge alone; loops are bailed earlier.
            return true;
        }
        let succs = block.successors(context);
        let ans = if succs.is_empty() {
            hit
        } else {
            succs.iter().all(|s| {
                check(
                    context,
                    s.block,
                    hit,
                    obligation_blocks,
                    memo,
                    visiting,
                )
            })
        };
        visiting.remove(&key);
        memo.insert(key, ans);
        ans
    }

    let mut memo = FxHashMap::default();
    let mut visiting = FxHashSet::default();
    check(
        context,
        start,
        false,
        obligation_blocks,
        &mut memo,
        &mut visiting,
    )
}

/// Earliest insert point in `d` that still dominates every obligation: the
/// first mid-block force inside `d`, else `d`'s terminator.
fn coalesce_placement(context: &Context, d: Block, obligations: &[Obligation]) -> Value {
    let insts: Vec<Value> = d.instruction_iter(context).collect();
    let mut best: Option<(usize, Value)> = None;
    for o in obligations {
        if obligation_block(context, *o) != d {
            continue;
        }
        let v = match o {
            Obligation::BeforeInst(inst) => *inst,
            Obligation::BeforeTerminator(_) => *insts.last().expect("terminator"),
        };
        let pos = insts.iter().position(|i| *i == v).unwrap_or(usize::MAX);
        if best.is_none_or(|(bp, _)| pos < bp) {
            best = Some((pos, v));
        }
    }
    best.map(|(_, v)| v).unwrap_or_else(|| {
        *insts.last().expect("block must have a terminator")
    })
}

/// Recompute delegated-read rewrites assuming materialization at each point in
/// `placements` (Pending until one of those instructions, Materialized after).
fn rescan_rewrites_with_placements(
    context: &Context,
    candidate: &Candidate,
    block_accesses: &FxHashMap<Block, Vec<(Value, Vec<Access>)>>,
    rpo: &[Block],
    entry: Block,
    reachable: &FxHashSet<Block>,
    placements: &[Value],
    plan: &mut Plan,
) -> bool {
    let placement_set: FxHashSet<Value> = placements.iter().copied().collect();
    let mut out_state: FxHashMap<Block, State> = FxHashMap::default();
    for _ in 0..(rpo.len() * 4 + 16) {
        let mut changed = false;
        for block in rpo {
            let mut preds = block
                .pred_iter(context)
                .filter(|p| reachable.contains(p))
                .peekable();
            let state_in = if *block == entry || preds.peek().is_none() {
                State::NoCopy
            } else {
                let mut states = FxIndexSet::default();
                for pred in preds {
                    states.insert(out_state.get(pred).copied().unwrap_or(State::NoCopy));
                }
                if states.contains(&State::Pending) && states.len() > 1 {
                    State::Materialized
                } else {
                    states.first().copied().unwrap_or(State::NoCopy)
                }
            };
            let Some(out) = transfer_with_placements(
                context,
                candidate,
                block_accesses,
                *block,
                state_in,
                &placement_set,
                None,
            ) else {
                return false;
            };
            if out_state.get(block).copied() != Some(out) {
                out_state.insert(*block, out);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for block in rpo {
        let mut preds = block
            .pred_iter(context)
            .filter(|p| reachable.contains(p))
            .peekable();
        let state_in = if *block == entry || preds.peek().is_none() {
            State::NoCopy
        } else {
            let mut states = FxIndexSet::default();
            for pred in preds {
                states.insert(out_state.get(pred).copied().unwrap_or(State::NoCopy));
            }
            if states.contains(&State::Pending) && states.len() > 1 {
                State::Materialized
            } else {
                states.first().copied().unwrap_or(State::NoCopy)
            }
        };
        if transfer_with_placements(
            context,
            candidate,
            block_accesses,
            *block,
            state_in,
            &placement_set,
            Some(plan),
        )
        .is_none()
        {
            return false;
        }
    }
    true
}

fn transfer_with_placements(
    context: &Context,
    candidate: &Candidate,
    block_accesses: &FxHashMap<Block, Vec<(Value, Vec<Access>)>>,
    block: Block,
    in_state: State,
    placements: &FxHashSet<Value>,
    mut plan: Option<&mut Plan>,
) -> Option<State> {
    let mut state = in_state;
    let mut after_candidate = block != candidate.block && in_state != State::NoCopy;
    for inst in block.instruction_iter(context) {
        if inst == candidate.inst {
            after_candidate = true;
            state = State::Pending;
            continue;
        }
        if block == candidate.block && !after_candidate {
            continue;
        }
        if placements.contains(&inst) && state == State::Pending {
            state = State::Materialized;
            continue;
        }
        let Some((_, accesses)) = block_accesses
            .get(&block)
            .and_then(|insts| insts.iter().find(|(i, _)| *i == inst))
        else {
            continue;
        };
        let has_forcing = accesses.iter().any(|a| {
            matches!(
                a.kind,
                AccessKind::PartialWrite | AccessKind::FullWrite | AccessKind::MayWrite
            )
        });
        if state == State::Pending && has_forcing {
            // A coalesced placement should dominate every force in its cluster.
            state = State::Materialized;
            continue;
        }
        if state == State::Pending {
            for access in accesses {
                if access.sym == candidate.dst_sym && access.kind == AccessKind::Read {
                    analyze_ptr_chain(context, access.ptr, candidate.dst_local)?;
                    if let Some(plan) = plan.as_mut() {
                        plan.rewrites.push((inst, access.ptr));
                        plan.delegated_reads += 1;
                    }
                }
            }
        }
    }
    Some(state)
}

/// Coalesce `obligations` to their ncd when anticipated; otherwise partition by
/// dominator-tree children of that ncd and retry each subset.
fn coalesce_obligation_subset(
    context: &Context,
    candidate: &Candidate,
    obligations: &[Obligation],
    dom_tree: &DomTree,
    all_blocks: &[Block],
    entry: Block,
) -> Vec<ClusterCoalesce> {
    if obligations.len() <= 1 {
        return Vec::new();
    }
    let ob_blocks: Vec<Block> = obligations
        .iter()
        .map(|o| obligation_block(context, *o))
        .collect();
    if ob_blocks
        .iter()
        .any(|b| !dom_tree.dominates(candidate.block, *b) && *b != candidate.block)
    {
        return Vec::new();
    }
    let Some(d) = nearest_common_dominator(dom_tree, &ob_blocks, all_blocks, entry) else {
        return Vec::new();
    };
    if !dom_tree.dominates(candidate.block, d) && d != candidate.block {
        return Vec::new();
    }
    let ob_block_set: FxHashSet<Block> = ob_blocks.iter().copied().collect();
    if is_anticipated(context, d, &ob_block_set) {
        let placement = coalesce_placement(context, d, obligations);
        let keep_candidate =
            candidate.block == d && candidate_already_at_force(context, candidate, placement);
        return vec![ClusterCoalesce {
            obligations: obligations.to_vec(),
            placement,
            keep_candidate,
        }];
    }
    // Full-set hoist is not anticipated (e.g. side exit from `d`). Try each
    // dominator-tree child as its own subset — a new D' per region.
    let mut out = Vec::new();
    for child in dom_tree.children(d) {
        let sub: Vec<Obligation> = obligations
            .iter()
            .copied()
            .filter(|o| {
                let b = obligation_block(context, *o);
                b == child || dom_tree.dominates(child, b)
            })
            .collect();
        out.extend(coalesce_obligation_subset(
            context,
            candidate,
            &sub,
            dom_tree,
            all_blocks,
            entry,
        ));
    }
    out
}

/// Try to replace sunk obligations with fewer copies at anticipated NCDs,
/// including per-region clusters when the global NCD is not anticipated.
fn try_coalesce_obligations(
    context: &Context,
    function: Function,
    candidate: &Candidate,
    plan: &Plan,
    dom_tree: &DomTree,
    reachable: &FxHashSet<Block>,
) -> Option<Coalesce> {
    let obligations: Vec<Obligation> = plan
        .inserts
        .iter()
        .copied()
        .map(Obligation::BeforeInst)
        .chain(
            plan.edge_inserts
                .iter()
                .copied()
                .map(Obligation::BeforeTerminator),
        )
        .collect();
    assert_well_built_obligations(context, dom_tree, &obligations);
    if obligations.len() <= 1 {
        return None;
    }

    let entry = function.get_entry_block(context);
    let all_blocks: Vec<Block> = function
        .block_iter(context)
        .filter(|b| reachable.contains(b))
        .collect();
    let clusters = coalesce_obligation_subset(
        context,
        candidate,
        &obligations,
        dom_tree,
        &all_blocks,
        entry,
    );
    if clusters.is_empty() {
        return None;
    }

    // Entire set collapses to the candidate site: leave IR unchanged.
    if clusters.len() == 1
        && clusters[0].keep_candidate
        && clusters[0].obligations.len() == obligations.len()
    {
        return Some(Coalesce::KeepOriginal);
    }

    let mut absorbed = Vec::new();
    let mut before = Vec::new();
    for cluster in clusters {
        if cluster.keep_candidate {
            // Subset keep-candidate under a partial coalesce is not expected;
            // treat as placing at the candidate force site without deleting it.
            continue;
        }
        absorbed.extend(cluster.obligations);
        before.push(cluster.placement);
    }
    if absorbed.is_empty() {
        return None;
    }
    Some(Coalesce::Places { absorbed, before })
}

fn obligation_absorbed(ob: Obligation, absorbed: &[Obligation]) -> bool {
    absorbed.iter().any(|a| match (a, ob) {
        (Obligation::BeforeInst(x), Obligation::BeforeInst(y)) => *x == y,
        (Obligation::BeforeTerminator(x), Obligation::BeforeTerminator(y)) => *x == y,
        _ => false,
    })
}

/// Try to push a single candidate copy down. Returns `Ok(true)` if the IR was
/// modified.
fn push_down_candidate(
    context: &mut Context,
    function: Function,
    candidate: &Candidate,
) -> Result<bool, IrError> {
    let blocks: Vec<Block> = function.block_iter(context).collect();
    let post_order = compute_post_order(context, &function);
    let reachable: FxHashSet<Block> = post_order.po_to_block.iter().copied().collect();
    if !reachable.contains(&candidate.block) {
        return Ok(false);
    }
    let rpo: Vec<Block> = post_order.po_to_block.iter().rev().copied().collect();

    // Classify every instruction once. Any unclassifiable interaction with
    // either symbol kills the candidate.
    let mut block_accesses: FxHashMap<Block, Vec<(Value, Vec<Access>)>> = FxHashMap::default();
    for block in &blocks {
        let mut insts = Vec::new();
        for inst in block.instruction_iter(context) {
            if inst == candidate.inst {
                continue;
            }
            match classify_inst(context, inst, candidate.dst_sym, candidate.src_sym) {
                Some(accesses) => insts.push((inst, accesses)),
                None => return Ok(false),
            }
        }
        block_accesses.insert(*block, insts);
    }

    // A mutable use (mutable call argument, asm operand) of either pointer
    // must not be able to execute before the candidate copy: a pointer
    // handed out before the copy exists could be used to write through an
    // alias we cannot track during the pending window. Mutable uses after
    // the candidate are fine -- they force materialization (see below).
    {
        let dom_tree = compute_dom_tree_from_po(context, function, &post_order);
        let candidate_pos = candidate
            .block
            .instruction_iter(context)
            .position(|inst| inst == candidate.inst)
            .unwrap_or(usize::MAX);
        for block in &blocks {
            let insts = &block_accesses[block];
            for (idx, (_, accesses)) in insts.iter().enumerate() {
                if !accesses.iter().any(|a| a.kind == AccessKind::MayWrite) {
                    continue;
                }
                let after_candidate = if *block == candidate.block {
                    // `insts` omits the candidate itself, so an entry at
                    // index `candidate_pos` or later follows the candidate.
                    idx >= candidate_pos
                } else {
                    dom_tree.dominates(candidate.block, *block)
                };
                if !after_candidate {
                    return Ok(false);
                }
            }
        }
    }

    // Transfer function: scan one block under an in-state, optionally
    // recording the plan.
    let scan = |block: Block, in_state: State, plan: &mut Option<&mut Plan>| -> Option<State> {
        let mut state = in_state;
        for (inst, accesses) in &block_accesses[&block] {
            let has_forcing = accesses.iter().any(|a| {
                matches!(
                    a.kind,
                    AccessKind::PartialWrite | AccessKind::FullWrite | AccessKind::MayWrite
                )
            });
            if state == State::Pending && has_forcing {
                let full_dst_overwrite = accesses.iter().all(|a| {
                    a.kind == AccessKind::Read
                        || (a.sym == candidate.dst_sym && a.kind == AccessKind::FullWrite)
                }) && accesses
                    .iter()
                    .any(|a| a.sym == candidate.dst_sym && a.kind == AccessKind::FullWrite);
                if !full_dst_overwrite {
                    if let Some(plan) = plan.as_deref_mut() {
                        if candidate_already_at_force(context, candidate, *inst) {
                            plan.keep_candidate = true;
                        } else {
                            plan.inserts.push(*inst);
                            if block != candidate.block {
                                plan.off_block_materializations += 1;
                            }
                        }
                    }
                }
                state = State::Materialized;
                continue;
            }
            if state == State::Pending {
                // Only reads remain: delegate destination reads to the source.
                for access in accesses {
                    if access.sym == candidate.dst_sym && access.kind == AccessKind::Read {
                        analyze_ptr_chain(context, access.ptr, candidate.dst_local)?;
                        if let Some(plan) = plan.as_deref_mut() {
                            plan.rewrites.push((*inst, access.ptr));
                            plan.delegated_reads += 1;
                        }
                    }
                }
            }
        }
        Some(state)
    };

    // Blocks containing the candidate need an ordered two-phase scan.
    let scan_block =
        |block: Block, in_state: State, plan: &mut Option<&mut Plan>| -> Option<State> {
            if block != candidate.block {
                return scan(block, in_state, plan);
            }
            // Split the block's accesses at the candidate instruction.
            let mut state = in_state;
            let mut after_candidate = false;
            let insts: Vec<Value> = block.instruction_iter(context).collect();
            for inst in insts {
                if inst == candidate.inst {
                    after_candidate = true;
                    state = State::Pending;
                    continue;
                }
                if !after_candidate {
                    continue;
                }
                let accesses = &block_accesses[&block]
                    .iter()
                    .find(|(i, _)| *i == inst)
                    .unwrap()
                    .1;
                let has_forcing = accesses.iter().any(|a| {
                    matches!(
                        a.kind,
                        AccessKind::PartialWrite | AccessKind::FullWrite | AccessKind::MayWrite
                    )
                });
                if state == State::Pending && has_forcing {
                    let full_dst_overwrite = accesses.iter().all(|a| {
                        a.kind == AccessKind::Read
                            || (a.sym == candidate.dst_sym && a.kind == AccessKind::FullWrite)
                    }) && accesses
                        .iter()
                        .any(|a| a.sym == candidate.dst_sym && a.kind == AccessKind::FullWrite);
                    if !full_dst_overwrite {
                        if let Some(plan) = plan.as_deref_mut() {
                            if candidate_already_at_force(context, candidate, inst) {
                                plan.keep_candidate = true;
                            } else {
                                plan.inserts.push(inst);
                            }
                        }
                    }
                    state = State::Materialized;
                    continue;
                }
                if state == State::Pending {
                    for access in accesses {
                        if access.sym == candidate.dst_sym && access.kind == AccessKind::Read {
                            analyze_ptr_chain(context, access.ptr, candidate.dst_local)?;
                            if let Some(plan) = plan.as_deref_mut() {
                                plan.rewrites.push((inst, access.ptr));
                                plan.delegated_reads += 1;
                            }
                        }
                    }
                }
            }
            Some(state)
        };

    // Blocks from which an access to the destination is still reachable.
    // Joins outside this set have a dead state: reconciling them would only
    // insert copies nobody can observe.
    let mut needed: FxHashSet<Block> = FxHashSet::default();
    {
        let mut stack: Vec<Block> = blocks
            .iter()
            .filter(|b| {
                block_accesses[b]
                    .iter()
                    .any(|(_, accesses)| accesses.iter().any(|a| a.sym == candidate.dst_sym))
            })
            .copied()
            .collect();
        while let Some(block) = stack.pop() {
            if needed.insert(block) {
                for pred in block.pred_iter(context) {
                    stack.push(*pred);
                }
            }
        }
    }

    // Forward dataflow over the CFG.
    let mut in_state: FxHashMap<Block, (State, bool)> = FxHashMap::default();
    let mut out_state: FxHashMap<Block, State> = FxHashMap::default();
    let mut forced: FxHashSet<Block> = FxHashSet::default();
    let entry = function.get_entry_block(context);
    for _round in 0..(blocks.len() * 4 + 16) {
        let mut changed = false;
        let mut new_forced: Vec<Block> = Vec::new();
        for block in &rpo {
            // Meet over predecessors.
            let mut preds = block
                .pred_iter(context)
                .filter(|p| reachable.contains(p))
                .peekable();
            let (state_in, reconcile) = if *block == entry || preds.peek().is_none() {
                (State::NoCopy, false)
            } else {
                let mut states = FxIndexSet::default();
                for pred in preds {
                    states.insert(out_state.get(pred).copied().unwrap_or(State::NoCopy));
                }
                if states.len() == 1 {
                    (states[0], false)
                } else if states.contains(&State::Pending) {
                    (State::Materialized, true)
                } else {
                    (State::Materialized, false)
                }
            };
            if reconcile && needed.contains(block) {
                for pred in block.pred_iter(context).filter(|p| reachable.contains(p)) {
                    if out_state.get(pred).copied() == Some(State::Pending)
                        && !forced.contains(pred)
                    {
                        new_forced.push(*pred);
                    }
                }
            }
            let prev_in = in_state.get(block).copied();
            if prev_in != Some((state_in, reconcile)) {
                in_state.insert(*block, (state_in, reconcile));
                changed = true;
            }
            let mut no_plan: Option<&mut Plan> = None;
            let Some(out) = scan_block(*block, state_in, &mut no_plan) else {
                return Ok(false);
            };
            let out = if forced.contains(block) {
                State::Materialized
            } else {
                out
            };
            if out_state.get(block).copied() != Some(out) {
                out_state.insert(*block, out);
                changed = true;
            }
        }
        for pred in new_forced {
            if forced.insert(pred) {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // The pending region must not cross a CFG cycle: a copy inserted inside a
    // loop body would be re-executed every iteration. Mid-block Pending is
    // invisible to the in/out check, so a candidate that itself lives in a
    // cycle is also rejected. The join at a loop header can further "hide"
    // pending by reconciling to Materialized and forcing a preheader edge
    // copy -- the pass bails on that so a sunk copy can never ride a backedge.
    let cycles = cycle_blocks(context, function);
    if cycles.contains(&candidate.block) {
        return Ok(false);
    }
    for block in &rpo {
        let pending_in = in_state.get(block).map(|(s, _)| *s) == Some(State::Pending);
        let pending_out = out_state.get(block).copied() == Some(State::Pending);
        if cycles.contains(block) && (pending_in || pending_out) {
            return Ok(false);
        }
    }
    for block in &rpo {
        if !forced.contains(block) {
            continue;
        }
        if block
            .successors(context)
            .iter()
            .any(|succ| cycles.contains(&succ.block))
        {
            return Ok(false);
        }
        // A forced pred with multiple successors is a critical-edge join:
        // inserting at the pred terminator would also run on edges that do
        // not need the copy. This pass does not split critical edges, so bail.
        if block.successors(context).len() > 1 {
            return Ok(false);
        }
    }

    // Build the final plan.
    let mut keep_candidate = false;
    let mut edge_inserts = Vec::new();
    for block in &rpo {
        if !forced.contains(block) {
            continue;
        }
        let terminator = block
            .instruction_iter(context)
            .next_back()
            .expect("block must have a terminator");
        if candidate.block == *block && candidate_already_at_force(context, candidate, terminator) {
            // A previous rewrite already parked the copy on this edge.
            keep_candidate = true;
        } else {
            edge_inserts.push(*block);
        }
    }
    let mut plan = Plan {
        rewrites: Vec::new(),
        inserts: Vec::new(),
        edge_inserts,
        delegated_reads: 0,
        off_block_materializations: forced.iter().filter(|b| **b != candidate.block).count(),
        keep_candidate,
    };
    for block in &rpo {
        let (state_in, _) = in_state
            .get(block)
            .copied()
            .unwrap_or((State::NoCopy, false));
        let mut plan_ref: Option<&mut Plan> = Some(&mut plan);
        if scan_block(*block, state_in, &mut plan_ref).is_none() {
            return Ok(false);
        }
    }

    // Coalesce multiple sunk obligations to their nearest common dominator when
    // every path from that point still hits an obligation (see module docs).
    let dom_tree = compute_dom_tree_from_po(context, function, &post_order);
    match try_coalesce_obligations(
        context,
        function,
        candidate,
        &plan,
        &dom_tree,
        &reachable,
    ) {
        Some(Coalesce::KeepOriginal) => {
            // One copy at the candidate is better than several sunk sites.
            return Ok(false);
        }
        Some(Coalesce::Places { absorbed, before }) => {
            plan.inserts
                .retain(|inst| !obligation_absorbed(Obligation::BeforeInst(*inst), &absorbed));
            plan.edge_inserts
                .retain(|block| !obligation_absorbed(Obligation::BeforeTerminator(*block), &absorbed));
            plan.inserts.extend(before.iter().copied());
            plan.rewrites.clear();
            plan.delegated_reads = 0;
            plan.keep_candidate = false;
            plan.off_block_materializations = 0;
            for inst in &plan.inserts {
                if inst.get_instruction(context).unwrap().parent != candidate.block {
                    plan.off_block_materializations += 1;
                }
            }
            for block in &plan.edge_inserts {
                if *block != candidate.block {
                    plan.off_block_materializations += 1;
                }
            }
            let mut placements = plan.inserts.clone();
            for block in &plan.edge_inserts {
                if let Some(term) = block.instruction_iter(context).next_back() {
                    placements.push(term);
                }
            }
            if !rescan_rewrites_with_placements(
                context,
                candidate,
                &block_accesses,
                &rpo,
                entry,
                &reachable,
                &placements,
                &mut plan,
            ) {
                return Ok(false);
            }
        }
        None => {}
    }

    // Profitability: at least one read must be delegated, or the copy must
    // move (later in the same block, to another block, or onto a join edge).
    // `keep_candidate` alone is not a transformation (IR unchanged).
    if plan.delegated_reads == 0 && plan.inserts.is_empty() && plan.edge_inserts.is_empty() {
        return Ok(false);
    }

    // Apply the plan.
    let insert_copy_before = |context: &mut Context, before: Value| {
        let dst_ptr = InstructionInserter::before(context, before).get_local(candidate.dst_local);
        let src_ptr = match candidate.src {
            SrcPtr::Local(local) => InstructionInserter::before(context, before).get_local(local),
            SrcPtr::Arg(val) => val,
        };
        InstructionInserter::before(context, before).mem_copy_val(dst_ptr, src_ptr);
    };

    for before in &plan.inserts {
        insert_copy_before(context, *before);
    }
    for pred in &plan.edge_inserts {
        let terminator = pred
            .instruction_iter(context)
            .next_back()
            .expect("block must have a terminator");
        insert_copy_before(context, terminator);
    }
    for (inst, old_ptr) in &plan.rewrites {
        let chain = analyze_ptr_chain(context, *old_ptr, candidate.dst_local)
            .expect("chain checked during planning");
        fn build(context: &mut Context, inst: Value, chain: &PtrChain, cur: Value) -> Value {
            match chain {
                PtrChain::Base => cur,
                PtrChain::Gep(inner, elem_ty, indices) => {
                    let base = build(context, inst, inner, cur);
                    InstructionInserter::before(context, inst).get_elem_ptr(
                        base,
                        *elem_ty,
                        indices.clone(),
                    )
                }
                PtrChain::Cast(inner, ty) => {
                    let base = build(context, inst, inner, cur);
                    InstructionInserter::before(context, inst).cast_ptr(base, *ty)
                }
            }
        }
        let mut cur = match candidate.src {
            SrcPtr::Local(local) => InstructionInserter::before(context, *inst).get_local(local),
            SrcPtr::Arg(val) => val,
        };
        cur = build(context, *inst, &chain, cur);
        inst.replace_instruction_value(context, *old_ptr, cur);
    }

    if !plan.keep_candidate {
        function.remove_instructions(context, |v| v == candidate.inst);
    }
    Ok(true)
}

pub fn memcpy_push_down(
    context: &mut Context,
    _analyses: &AnalysisResults,
    function: Function,
) -> Result<bool, IrError> {
    let mut modified = false;
    // Restart after each successful rewrite: classifications are stale as
    // soon as the IR changes. Each rewrite removes one candidate copy, so
    // this always terminates.
    for _ in 0..64 {
        let candidates = find_candidates(context, function);
        let mut progressed = false;
        for candidate in &candidates {
            if push_down_candidate(context, function, candidate)? {
                modified = true;
                progressed = true;
                break;
            }
        }
        if !progressed {
            break;
        }
    }
    Ok(modified)
}
