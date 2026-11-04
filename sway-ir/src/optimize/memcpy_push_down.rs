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
//! **Never-worse (pathwise):** every transform keeps per-path `mem_copy_val`
//! count ≤ the original candidate (sites are not comparable, so a path hits
//! at most one). More *static* copies are allowed only when some exit path
//! sheds the copy entirely (side-exit / cluster sinks).
//!
//! # Glossary
//!
//! Lattice / CFG analysis:
//!
//! * **NoCopy** — this path never has the candidate copy (killed or never started).
//! * **Pending** — `dst` still aliased to `src` (logical copy; bytes may not be private).
//! * **Materialized** — `dst` is an independent private buffer on this path.
//! * **uses_dst** — a use of `dst` is still reachable from this block.
//! * **in_states_disagree** — at a join, predecessor out-states are not all equal
//!   (especially when one is still Pending).
//! * **must_materialize_before_exit** — this block must emit a real `mem_copy_val`
//!   before its terminator (the set of **pending_pred**s of a live join). After
//!   dataflow, dominated members are pruned: materializing at a dominator
//!   already covers paths through a dominated pending_pred.
//! * **pending_pred** — join predecessor whose out-state is still Pending.
//! * **multi_succ_pending_pred** — a `pending_pred` with more than one successor
//!   (insert-before-terminator would fan out; often a critical edge into the join).
//!
//! Candidate / plan / coalesce:
//!
//! * **candidate** — the `mem_copy_val dst, src` instruction under consideration.
//! * **CandidateSite** — a sunk *demand* in set `O`: where a materializing copy
//!   would be inserted (`BeforeInst` or `BeforeTerminator`). Not the same as
//!   **candidate**.
//! * **`O`** — the set of all `CandidateSite`s from sunk analysis for one candidate.
//! * **`D` / `ncd(O)`** — nearest common dominator of every site in `O`.
//! * **every_path_needs_copy** — every CFG path from `D` to a function exit
//!   (`ret` / `revert` / other no-successor terminator) visits at least one
//!   block that owns a site in `O`; hoist profitability check.
//! * **RetainCandidate** — coalesce chooses to leave the original candidate in
//!   place rather than sink/re-insert (typically when `D` is already that site).
//! * **Places / absorbed** — coalesce replaces some sunk sites with fewer
//!   insert-before points; absorbed sites are removed from the sunk plan.
//! * **cluster** — a subset of `O` coalesced independently when the full-set
//!   `ncd` fails `every_path_needs_copy` (partition by dominator-tree children).
//!
//! Access classification (what an instruction does to `dst` / `src`):
//!
//! * **Read** — load through the pointer (may be delegated to `src` while Pending).
//! * **PartialWrite** — write that does not cover the whole symbol (forces materialize).
//! * **FullWrite** — write covering the whole symbol (may kill the copy on that path).
//! * **MayWrite** — opaque use (mutable call / asm) that may read and write.
//!
//! # Join materialization (soundness)
//!
//! Marking only the **immediate** `pending_pred`s of a live join
//! (`in_states_disagree` && `uses_dst`) is enough for correctness. The copy is
//! inserted on the incoming edge (before that predecessor's terminator).
//! Ancestors above that edge can stay Pending.
//!
//! Related fixtures:
//! [`sound_join_after_src_mutation.ir`](../../tests/memcpy_push_down/sound_join_after_src_mutation.ir),
//! [`sound_join_pending_nocopy_live.ir`](../../tests/memcpy_push_down/sound_join_pending_nocopy_live.ir).
//!
//! # CandidateSites and hoisting (profitability)
//!
//! Sunk dataflow records the set `O` of **CandidateSite**s:
//!
//! - before a forcing instruction (`Plan::inserts`);
//! - before a `must_materialize_before_exit` predecessor terminator
//!   (`Plan::edge_inserts`).
//!
//! If `O` is empty, the win is full elimination / delegate-only. Otherwise let
//! `D = ncd(O)`. Hoist to `D` only when **`every_path_needs_copy(D)`**. If not,
//! partition `O` by dominator-tree children of `D` and retry each subset
//! recursively (clusters). Invariants checked with `assert!`: well-built `O`
//! (no comparable sites), sites under the candidate, undominated
//! `must_materialize_before_exit` after prune, coalesce placement dominates
//! its cluster, and no unexpected subset `keep_candidate`.
//!
//! A **`multi_succ_pending_pred`** is special: insert-before-terminator fans out
//! to every successor. The pass records it in `O` (and may coalesce it away).
//! If the edge insert remains, it is applied only when every successor path
//! still needs `dst` (`every_path_needs_copy` over `uses_dst`); otherwise the
//! pass bails rather than splitting the critical edge.
//!
//! # Worked examples (step-by-step in the `.ir` comments)
//!
//! * [`limit_critical_edge_live_join.ir`](../../tests/memcpy_push_down/limit_critical_edge_live_join.ir)
//!   — `multi_succ_pending_pred` at entry; `keep_candidate` → IR unchanged.
//! * [`write_inside_if_no_else_read_after.ir`](../../tests/memcpy_push_down/write_inside_if_no_else_read_after.ir)
//!   — same Sway with fallthrough split; `every_path_needs_copy(entry)` →
//!   **RetainCandidate**.
//! * [`write_inside_if_read_inside_else_join.ir`](../../tests/memcpy_push_down/write_inside_if_read_inside_else_join.ir)
//!   — join not in `uses_dst` for `order_copy` (dead join); read arm delegates.
//! * [`limit_hoist_side_exit_keeps_sunk.ir`](../../tests/memcpy_push_down/limit_hoist_side_exit_keeps_sunk.ir)
//!   — side exit: full `ncd` fails `every_path_needs_copy`; singleton clusters
//!   leave sunk arm copies.
//! * [`opt_hoist_two_clusters_side_exit.ir`](../../tests/memcpy_push_down/opt_hoist_two_clusters_side_exit.ir)
//!   — side exit but two diamonds: recursive clusters → one copy per region.
//! * [`opt_hoist_nested_cluster_under_side_exit.ir`](../../tests/memcpy_push_down/opt_hoist_nested_cluster_under_side_exit.ir)
//!   — cold exit off entry; nested if under hot → one copy at `block_hot`.
//! * [`opt_critical_edge_side_exit_before_if.ir`](../../tests/memcpy_push_down/opt_critical_edge_side_exit_before_if.ir)
//!   — cold exit + `multi_succ_pending_pred` on hot; coalesce PlaceBefore at
//!   `block_hot` (cold stays copy-free).
//! * [`opt_nested_join_prune_dominated_pending_pred.ir`](../../tests/memcpy_push_down/opt_nested_join_prune_dominated_pending_pred.ir)
//!   — nested joins: keep only undominated `must_materialize_before_exit`
//!   sites; sink to outer `multi_succ_pending_pred`, cold copy-free.
//! * [`opt_nested_join_retain_over_dominated_pending_pred.ir`](../../tests/memcpy_push_down/opt_nested_join_retain_over_dominated_pending_pred.ir)
//!   — same nesting without cold exit; prune dominated pending_pred →
//!   **keep_candidate**.
//!

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
enum CandidateSite {
    /// Mid-block force: copy immediately before this instruction.
    BeforeInst(Value),
    /// Live in_states_disagree edge: copy immediately before this block's terminator.
    BeforeTerminator(Block),
}

enum Coalesce {
    /// Leave the candidate where it is; do not apply the sunk plan.
    RetainCandidate,
    /// Replace the absorbed sunk candidate_sites with these insert-before points.
    /// CandidateSites not listed stay at their sunk sites.
    Places {
        absorbed: Vec<CandidateSite>,
        before: Vec<Value>,
    },
}

struct ClusterCoalesce {
    sites: Vec<CandidateSite>,
    placement: Value,
    keep_candidate: bool,
}

fn candidate_site_block(context: &Context, ob: CandidateSite) -> Block {
    match ob {
        CandidateSite::BeforeInst(inst) => inst.get_instruction(context).unwrap().parent,
        CandidateSite::BeforeTerminator(block) => block,
    }
}

fn candidate_site_insert_point(context: &Context, ob: CandidateSite) -> Value {
    match ob {
        CandidateSite::BeforeInst(inst) => inst,
        CandidateSite::BeforeTerminator(block) => block
            .instruction_iter(context)
            .next_back()
            .expect("block must have a terminator"),
    }
}

fn plan_candidate_sites(plan: &Plan) -> Vec<CandidateSite> {
    plan.inserts
        .iter()
        .copied()
        .map(CandidateSite::BeforeInst)
        .chain(
            plan.edge_inserts
                .iter()
                .copied()
                .map(CandidateSite::BeforeTerminator),
        )
        .collect()
}

/// Sunk `O` must not contain two sites where one dominates the other: the
/// earlier materialization would already have made that path Materialized.
fn assert_well_built_candidate_sites(
    context: &Context,
    dom_tree: &DomTree,
    sites: &[CandidateSite],
) {
    for (i, &a) in sites.iter().enumerate() {
        let pa = candidate_site_insert_point(context, a);
        for &b in &sites[i + 1..] {
            let pb = candidate_site_insert_point(context, b);
            assert!(
                pa != pb
                    && !dom_tree.dominates_instr(context, pa, pb)
                    && !dom_tree.dominates_instr(context, pb, pa),
                "well-built CandidateSite set must not contain comparable sites \
                 ({a:?} dominates or equals {b:?}, or vice versa)"
            );
        }
    }
}

/// Pending starts at the candidate, so every materialization site must lie
/// on a path dominated by the candidate block (or be that block).
fn assert_sites_under_candidate(
    context: &Context,
    dom_tree: &DomTree,
    candidate: &Candidate,
    sites: &[CandidateSite],
) {
    for &site in sites {
        let b = candidate_site_block(context, site);
        assert!(
            b == candidate.block || dom_tree.dominates(candidate.block, b),
            "CandidateSite {site:?} must be dominated by the candidate block"
        );
    }
}

/// Pruned `must_materialize_before_exit` must have no dominated pairs.
fn assert_undominated_blocks(dom_tree: &DomTree, blocks: &FxHashSet<Block>) {
    let blocks: Vec<Block> = blocks.iter().copied().collect();
    for (i, &a) in blocks.iter().enumerate() {
        for &b in &blocks[i + 1..] {
            assert!(
                !dom_tree.dominates(a, b) && !dom_tree.dominates(b, a),
                "must_materialize_before_exit must be undominated after prune \
                 ({a:?} and {b:?} are comparable)"
            );
        }
    }
}

/// A coalesced placement must dominate every absorbed site's insert point.
fn assert_placement_dominates_sites(
    context: &Context,
    dom_tree: &DomTree,
    placement: Value,
    sites: &[CandidateSite],
) {
    for &site in sites {
        let pb = candidate_site_insert_point(context, site);
        assert!(
            dom_tree.dominates_instr(context, placement, pb),
            "coalesce placement must dominate CandidateSite {site:?}"
        );
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
        if blocks.iter().all(|&b| dom_tree.dominates(cand, b))
            && (!found || dom_tree.dominates(best, cand))
        {
            best = cand;
            found = true;
        }
    }
    found.then_some(best)
}

/// True when every CFG path from `start` to a function exit visits at least
/// one block in `candidate_site_blocks`.
fn every_path_needs_copy(
    context: &Context,
    start: Block,
    candidate_site_blocks: &FxHashSet<Block>,
) -> bool {
    fn check(
        context: &Context,
        block: Block,
        hit: bool,
        candidate_site_blocks: &FxHashSet<Block>,
        memo: &mut FxHashMap<(Block, bool), bool>,
        visiting: &mut FxHashSet<(Block, bool)>,
    ) -> bool {
        let hit = hit || candidate_site_blocks.contains(&block);
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
            succs
                .iter()
                .all(|s| check(context, s.block, hit, candidate_site_blocks, memo, visiting))
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
        candidate_site_blocks,
        &mut memo,
        &mut visiting,
    )
}

/// Earliest insert point in `d` that still dominates every CandidateSite: the
/// first mid-block force inside `d`, else `d`'s terminator.
fn coalesce_placement(context: &Context, d: Block, sites: &[CandidateSite]) -> Value {
    let insts: Vec<Value> = d.instruction_iter(context).collect();
    let mut best: Option<(usize, Value)> = None;
    for o in sites {
        if candidate_site_block(context, *o) != d {
            continue;
        }
        let v = match o {
            CandidateSite::BeforeInst(inst) => *inst,
            CandidateSite::BeforeTerminator(_) => *insts.last().expect("terminator"),
        };
        let pos = insts.iter().position(|i| *i == v).unwrap_or(usize::MAX);
        if best.is_none_or(|(bp, _)| pos < bp) {
            best = Some((pos, v));
        }
    }
    let v = best
        .map(|(_, v)| v)
        .unwrap_or_else(|| *insts.last().expect("block must have a terminator"));
    assert_eq!(
        v.get_instruction(context).unwrap().parent,
        d,
        "coalesce placement must be an instruction of the NCD block"
    );
    v
}

/// Recompute delegated-read rewrites assuming materialization at each point in
/// `placements` (Pending until one of those instructions, Materialized after).
#[allow(clippy::too_many_arguments)]
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

/// Coalesce `sites` to their ncd when `every_path_needs_copy`; otherwise
/// partition by dominator-tree children of that ncd and retry each subset.
fn coalesce_candidate_site_subset(
    context: &Context,
    candidate: &Candidate,
    sites: &[CandidateSite],
    dom_tree: &DomTree,
    all_blocks: &[Block],
    entry: Block,
) -> Vec<ClusterCoalesce> {
    if sites.len() <= 1 {
        return Vec::new();
    }
    let site_blocks: Vec<Block> = sites
        .iter()
        .map(|o| candidate_site_block(context, *o))
        .collect();
    if site_blocks
        .iter()
        .any(|b| !dom_tree.dominates(candidate.block, *b) && *b != candidate.block)
    {
        return Vec::new();
    }
    let Some(d) = nearest_common_dominator(dom_tree, &site_blocks, all_blocks, entry) else {
        return Vec::new();
    };
    if !dom_tree.dominates(candidate.block, d) && d != candidate.block {
        return Vec::new();
    }
    let site_block_set: FxHashSet<Block> = site_blocks.iter().copied().collect();
    if every_path_needs_copy(context, d, &site_block_set) {
        let placement = coalesce_placement(context, d, sites);
        assert_placement_dominates_sites(context, dom_tree, placement, sites);
        let keep_candidate =
            candidate.block == d && candidate_already_at_force(context, candidate, placement);
        return vec![ClusterCoalesce {
            sites: sites.to_vec(),
            placement,
            keep_candidate,
        }];
    }
    // Full-set hoist is not every_path_needs_copy (e.g. side exit from `d`).
    // Try each dominator-tree child as its own subset — a new D' per region.
    let mut out = Vec::new();
    for child in dom_tree.children(d) {
        let sub: Vec<CandidateSite> = sites
            .iter()
            .copied()
            .filter(|o| {
                let b = candidate_site_block(context, *o);
                b == child || dom_tree.dominates(child, b)
            })
            .collect();
        out.extend(coalesce_candidate_site_subset(
            context, candidate, &sub, dom_tree, all_blocks, entry,
        ));
    }
    out
}

/// Try to replace sunk CandidateSites with fewer copies at NCDs where
/// `every_path_needs_copy`, including per-region clusters when the global NCD
/// does not.
fn try_coalesce_candidate_sites(
    context: &Context,
    function: Function,
    candidate: &Candidate,
    plan: &Plan,
    dom_tree: &DomTree,
    reachable: &FxHashSet<Block>,
) -> Option<Coalesce> {
    let sites = plan_candidate_sites(plan);
    assert_well_built_candidate_sites(context, dom_tree, &sites);
    assert_sites_under_candidate(context, dom_tree, candidate, &sites);
    if sites.len() <= 1 {
        return None;
    }

    let entry = function.get_entry_block(context);
    let all_blocks: Vec<Block> = function
        .block_iter(context)
        .filter(|b| reachable.contains(b))
        .collect();
    let clusters =
        coalesce_candidate_site_subset(context, candidate, &sites, dom_tree, &all_blocks, entry);
    if clusters.is_empty() {
        return None;
    }

    // Entire set collapses to the candidate site: leave IR unchanged.
    if clusters.len() == 1 && clusters[0].keep_candidate && clusters[0].sites.len() == sites.len() {
        return Some(Coalesce::RetainCandidate);
    }

    let mut absorbed = Vec::new();
    let mut before = Vec::new();
    for cluster in clusters {
        // keep_candidate is only valid for the full-set RetainCandidate case
        // handled above; a partial/subset keep would silently drop sites.
        assert!(
            !cluster.keep_candidate,
            "keep_candidate only expected when the full site set coalesces to the candidate"
        );
        assert_placement_dominates_sites(context, dom_tree, cluster.placement, &cluster.sites);
        absorbed.extend(cluster.sites);
        before.push(cluster.placement);
    }
    if absorbed.is_empty() {
        return None;
    }
    Some(Coalesce::Places { absorbed, before })
}

fn candidate_site_absorbed(ob: CandidateSite, absorbed: &[CandidateSite]) -> bool {
    absorbed.iter().any(|a| match (a, ob) {
        (CandidateSite::BeforeInst(x), CandidateSite::BeforeInst(y)) => *x == y,
        (CandidateSite::BeforeTerminator(x), CandidateSite::BeforeTerminator(y)) => *x == y,
        _ => false,
    })
}

/// Drop `must_materialize_before_exit` blocks strictly dominated by another
/// member: a copy before the dominator's terminator already materializes every
/// path through the dominated block, so a second edge site would make `O`
/// ill-formed (comparable BeforeTerminators) or double-copy on that path.
fn prune_dominated_materialize_exits(
    dom_tree: &DomTree,
    blocks: &FxHashSet<Block>,
) -> FxHashSet<Block> {
    blocks
        .iter()
        .copied()
        .filter(|&b| !blocks.iter().any(|&a| a != b && dom_tree.dominates(a, b)))
        .collect()
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

    // Blocks from which an access to the destination is still reachable
    // (`uses_dst`). Joins outside this set have a dead state: marking
    // pending_preds there would only insert copies nobody can observe.
    let mut uses_dst: FxHashSet<Block> = FxHashSet::default();
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
            if uses_dst.insert(block) {
                for pred in block.pred_iter(context) {
                    stack.push(*pred);
                }
            }
        }
    }

    // Forward dataflow over the CFG.
    let mut in_state: FxHashMap<Block, (State, bool)> = FxHashMap::default();
    let mut out_state: FxHashMap<Block, State> = FxHashMap::default();
    let mut must_materialize_before_exit: FxHashSet<Block> = FxHashSet::default();
    let entry = function.get_entry_block(context);
    for _round in 0..(blocks.len() * 4 + 16) {
        let mut changed = false;
        let mut new_pending_preds: Vec<Block> = Vec::new();
        for block in &rpo {
            // Meet over predecessors.
            let mut preds = block
                .pred_iter(context)
                .filter(|p| reachable.contains(p))
                .peekable();
            let (state_in, in_states_disagree) = if *block == entry || preds.peek().is_none() {
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
            if in_states_disagree && uses_dst.contains(block) {
                for pred in block.pred_iter(context).filter(|p| reachable.contains(p)) {
                    if out_state.get(pred).copied() == Some(State::Pending)
                        && !must_materialize_before_exit.contains(pred)
                    {
                        new_pending_preds.push(*pred);
                    }
                }
            }
            let prev_in = in_state.get(block).copied();
            if prev_in != Some((state_in, in_states_disagree)) {
                in_state.insert(*block, (state_in, in_states_disagree));
                changed = true;
            }
            let mut no_plan: Option<&mut Plan> = None;
            let Some(out) = scan_block(*block, state_in, &mut no_plan) else {
                return Ok(false);
            };
            let out = if must_materialize_before_exit.contains(block) {
                State::Materialized
            } else {
                out
            };
            if out_state.get(block).copied() != Some(out) {
                out_state.insert(*block, out);
                changed = true;
            }
        }
        for pred in new_pending_preds {
            if must_materialize_before_exit.insert(pred) {
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
    // pending by in_states_disagree → Materialized and must_materialize_before_exit
    // on a preheader edge — the pass bails on that so a sunk copy can never
    // ride a backedge.
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
        if !must_materialize_before_exit.contains(block) {
            continue;
        }
        if block
            .successors(context)
            .iter()
            .any(|succ| cycles.contains(&succ.block))
        {
            return Ok(false);
        }
        // multi_succ_pending_pred: keep in O for coalesce; bail later only if
        // the edge insert would still be applied (see post-coalesce check).
    }

    // Nested joins can mark both a pending_pred and a dominated descendant.
    // Keep only undominated edge sites so O stays well-built and no path pays
    // for two copies. See `opt_nested_join_prune_dominated_pending_pred.ir`.
    let dom_tree = compute_dom_tree_from_po(context, function, &post_order);
    let must_materialize_before_exit =
        prune_dominated_materialize_exits(&dom_tree, &must_materialize_before_exit);
    assert_undominated_blocks(&dom_tree, &must_materialize_before_exit);

    // Build the final plan.
    let mut keep_candidate = false;
    let mut edge_inserts = Vec::new();
    for block in &rpo {
        if !must_materialize_before_exit.contains(block) {
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
        off_block_materializations: must_materialize_before_exit
            .iter()
            .filter(|b| **b != candidate.block)
            .count(),
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

    // Coalesce multiple sunk CandidateSites to their nearest common dominator
    // when every_path_needs_copy (see module docs).
    match try_coalesce_candidate_sites(context, function, candidate, &plan, &dom_tree, &reachable) {
        Some(Coalesce::RetainCandidate) => {
            // One copy at the candidate is better than several sunk sites.
            return Ok(false);
        }
        Some(Coalesce::Places { absorbed, before }) => {
            plan.inserts.retain(|inst| {
                !candidate_site_absorbed(CandidateSite::BeforeInst(*inst), &absorbed)
            });
            plan.edge_inserts.retain(|block| {
                !candidate_site_absorbed(CandidateSite::BeforeTerminator(*block), &absorbed)
            });
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
            // Coalesce must leave O well-built and under the candidate.
            let sites = plan_candidate_sites(&plan);
            assert_well_built_candidate_sites(context, &dom_tree, &sites);
            assert_sites_under_candidate(context, &dom_tree, candidate, &sites);
        }
        None => {}
    }

    // A remaining multi_succ_pending_pred edge insert fans out to every
    // successor. Allow that only when every successor path still needs `dst`
    // (fan-out is then both sound and not wasted on a free side exit).
    // Otherwise refuse rather than split the critical edge.
    // See `opt_critical_edge_side_exit_before_if.ir` (allowed) vs a side exit
    // off the multi_succ block itself (bail).
    for pred in &plan.edge_inserts {
        if pred.successors(context).len() > 1 {
            let fanout_ok = pred
                .successors(context)
                .iter()
                .all(|succ| every_path_needs_copy(context, succ.block, &uses_dst));
            if !fanout_ok {
                return Ok(false);
            }
        }
    }

    // Profitability: at least one read must be delegated, or the copy must
    // move (later in the same block, to another block, or onto a join edge).
    // `keep_candidate` alone is not a transformation (IR unchanged).
    if plan.delegated_reads == 0 && plan.inserts.is_empty() && plan.edge_inserts.is_empty() {
        return Ok(false);
    }

    // Never-worse (pathwise): replacing one candidate with several sunk sites
    // is only a win when some path from the candidate reaches an exit without
    // hitting any site (typically a side / early exit). Otherwise one copy at
    // the candidate dominates, and coalesce should have retained it — refuse
    // pure static size growth. See `limit_hoist_side_exit_keeps_sunk.ir`.
    if !plan.keep_candidate {
        let site_blocks: FxHashSet<Block> = plan
            .inserts
            .iter()
            .map(|inst| inst.get_instruction(context).unwrap().parent)
            .chain(plan.edge_inserts.iter().copied())
            .collect();
        if site_blocks.len() > 1 && every_path_needs_copy(context, candidate.block, &site_blocks) {
            return Ok(false);
        }
    }

    // Final O: still well-built and under the candidate before mutating IR.
    {
        let sites = plan_candidate_sites(&plan);
        assert_well_built_candidate_sites(context, &dom_tree, &sites);
        assert_sites_under_candidate(context, &dom_tree, candidate, &sites);
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
