//! `move_to_literal_pools` — relocate single-function data-section entries
//! into the using function's literal pool, lowering each address-of reference
//! to a single `$pc`-relative `ADDI` (4 bytes), shrinking the data section *and*
//! the code that takes addresses.
//!
//! See `PLAN.txt` for the full design. In short:
//!
//! - Each function may end with an [`AllocatedInstruction::LiteralPool`] op
//!   carrying a list of [`Entry`]s (the same type [`DataSection`] uses). At
//!   finalization the pool's bytes are emitted inline in the code section,
//!   `$pc`-relative to the function, and never executed (it sits after the
//!   function's return).
//! - This pass runs once, at program scope, *after* register allocation and
//!   the per-function allocated optimization, but *before* label resolution
//!   (`collect_far_jumps` / `resolve_labels`). Running it before label
//!   resolution is essential: relocating an address-of from `AddrFromDataSection`
//!   (8 bytes, `MOVI` + `ADD`) to `AddrFromLiteralPool` (4 bytes, `ADDI`)
//!   shrinks the code, so the shrunk sizes must be in place when offsets and
//!   far-jump decisions are computed. (Value loads `LoadFromDataSection` lower
//!   to a single `LW`/`LB` (4 bytes) whereas `LoadFromLiteralPool` lowers to 3
//!   instructions (12 bytes); relocating those is not size-favorable and is
//!   deferred.)
//!
//! ## Eligibility
//!
//! Only `non_configurable` entries are candidates; configurables must stay at
//! the very end of the binary. An entry is relocated only when **every** use of
//! it is an `AddrFromDataSection` (address-of) and all of those uses lie in a
//! **single** function. The prologue is scanned for uses too (so an entry also
//! referenced from the prologue — e.g. a selector value-load — is correctly seen
//! as ineligible), but is never itself a relocation target: a per-function pool
//! only reaches uses in that function, and cross-function uses keep their entry
//! in the shared data section.
//!
//! ## The 12-bit reach check (conservative, no revert)
//!
//! `ADDI dest, $pc, imm12` reaches a pool entry at most `TWELVE_BITS` (4095)
//! bytes *forward* of the use. A naive relocation can violate this: shrinking
//! intervening code shifts the pool, and appending entries to the pool grows
//! the offset of later entries. The pass avoids any revert/fixpoint by using a
//! conservative *worst-case* bound for the reach, computed as:
//!
//! ```text
//! reach = (pool_start - use_offset) - 4 + entry_offset
//! ```
//!
//! where `pool_start - use_offset` is measured with `worst_case_instruction_size`
//! for every op between the use and the pool (an upper bound on the actual
//! size), the `-4` replaces the use op's current worst case (`AddrFromDataSection`
//! = 8 bytes) with its relocated size (`ADDI` = 4 bytes), and `entry_offset` is
//! the byte offset of the entry within the pool (exact, since entries are
//! appended in data-section index order). Because every term is an upper bound
//! on its final value and later relocations only ever *shrink* the code (more
//! `ADDI`s) and only append pool entries *after* the current one, a use that
//! passes the check at decision time can never later exceed the bound. So a
//! single forward pass in data-section index order is correct, with no
//! iteration and no revert. The cost is conservativeness: some entries whose
//! actual reach would fit may be left in the data section.

use crate::asm_generation::fuel::allocated_abstract_instruction_set::AllocatedAbstractInstructionSet;
use crate::asm_generation::fuel::compiler_constants::TWELVE_BITS;
use crate::asm_generation::fuel::data_section::{
    DataId, DataIdEntryKind, DataSection, Entry, PoolEntryId,
};
use crate::asm_lang::allocated_ops::AllocatedInstruction;
use crate::asm_lang::AllocatedAbstractOp;
use either::Either;

/// A single reference to a [`DataId`] from the prologue (fn_idx 0) or some
/// function's ops (fn_idx 1.., indexing into the `functions` slice).
#[derive(Debug)]
struct UseSite {
    fn_idx: usize,
    op_idx: usize,
    is_addr: bool,
}

/// Run the pass once, at program scope, after allocation. `prologue` is scanned
/// for data references (so entries used from the prologue stay in the data
/// section) but is never a relocation target. `functions` are scanned and
/// mutated (references rewritten, pool entries appended). `data_section` has the
/// relocated entries tombstoned. See the module docs for the eligibility rules
/// and the conservative reach check.
pub(crate) fn move_to_literal_pools(
    prologue: &AllocatedAbstractInstructionSet,
    functions: &mut [AllocatedAbstractInstructionSet],
    data_section: &mut DataSection,
) {
    // Phase 1: collect every reference to each data-section entry, across the
    // prologue and all functions. Done with an immutable scan before any
    // mutation so the captured op indices stay valid (we only ever *append* a
    // pool op at a function's end and rewrite references in place; we never
    // insert or remove earlier ops).
    let mut uses: rustc_hash::FxHashMap<DataId, Vec<UseSite>> = rustc_hash::FxHashMap::default();
    collect_uses(0, &prologue.ops, &mut uses);
    for (i, func) in functions.iter().enumerate() {
        collect_uses(i + 1, &func.ops, &mut uses);
    }

    // Phase 2: decide and apply relocations. Iterate non-configurable entries
    // in index order so that pool layout (and thus bytecode) is deterministic
    // across builds, independent of the hash map's iteration order, and so that
    // a relocated entry is appended to the pool before any higher-indexed entry
    // (keeping per-entry pool offsets stable once decided).
    let non_config_count = data_section.non_configurables.len();
    for idx in 0..non_config_count {
        let id = DataId {
            idx: idx as u32,
            kind: DataIdEntryKind::NonConfigurable,
        };
        let Some(sites) = uses.get(&id) else {
            continue;
        };
        // Eligibility: every use is an address-of, and all uses are in a single
        // real function (fn_idx >= 1; fn_idx 0 is the prologue, never a target).
        // Configurables are skipped by construction: their DataId kind is
        // Configurable and they never appear in `non_configurables`.
        if sites.iter().any(|s| !s.is_addr) {
            continue;
        }
        let Some(target_fn1) = sites.first().map(|s| s.fn_idx) else {
            continue;
        };
        if target_fn1 == 0 {
            continue;
        }
        if !sites.iter().all(|s| s.fn_idx == target_fn1) {
            continue;
        }
        let target_fn = target_fn1 - 1;

        // Conservative reach check (see module docs). `entry_offset` is the byte
        // offset the entry would get within the function's pool (exact: entries
        // are appended in index order, so it equals the pool's current size).
        let (pool_idx, entry_offset) = match pool_state(&functions[target_fn].ops) {
            Some((idx, size)) => (idx, size),
            None => (functions[target_fn].ops.len(), 0),
        };
        let offsets = worst_case_byte_offsets(&functions[target_fn].ops);
        let pool_start = offsets[pool_idx];
        let all_in_range = sites.iter().all(|site| {
            let use_offset = offsets[site.op_idx];
            // `pool_start - use_offset` includes the use op's worst-case size
            // (8 bytes, as `AddrFromDataSection`); replace it with the relocated
            // `ADDI` size (4 bytes) by subtracting 4.
            let reach = pool_start - use_offset - 4 + entry_offset;
            reach <= TWELVE_BITS
        });
        if !all_in_range {
            continue;
        }

        // Relocate: copy the entry into the function's pool, rewrite every
        // reference to point at the new pool entry, and tombstone the data
        // entry.
        let entry = data_section.non_configurables[idx].clone();
        let pool_id = append_entry_to_pool(&mut functions[target_fn].ops, entry);
        for site in sites {
            let op = &mut functions[target_fn].ops[site.op_idx];
            if let Either::Left(AllocatedInstruction::AddrFromDataSection(reg, _)) = &op.opcode {
                op.opcode = Either::Left(AllocatedInstruction::AddrFromLiteralPool(
                    reg.clone(),
                    pool_id.clone(),
                ));
            } else {
                unreachable!("eligibility guarantees an address-of use");
            }
        }
        data_section.mark_relocated(&id);
    }
}

/// Record every `AddrFromDataSection`/`LoadFromDataSection` reference in `ops`
/// under `fn_idx` into `uses`.
fn collect_uses(
    fn_idx: usize,
    ops: &[AllocatedAbstractOp],
    uses: &mut rustc_hash::FxHashMap<DataId, Vec<UseSite>>,
) {
    for (op_idx, op) in ops.iter().enumerate() {
        let (id, is_addr) = match &op.opcode {
            Either::Left(AllocatedInstruction::AddrFromDataSection(_, id)) => (id.clone(), true),
            Either::Left(AllocatedInstruction::LoadFromDataSection(_, id)) => (id.clone(), false),
            _ => continue,
        };
        uses.entry(id).or_default().push(UseSite {
            fn_idx,
            op_idx,
            is_addr,
        });
    }
}

/// Worst-case byte offset of every op in `ops` (i.e. offset[i] = byte offset of
/// op i, offset[len] = total). Uses `worst_case_instruction_size`, an upper
/// bound on the actual emitted size, so the reach derived from these is a valid
/// upper bound too.
fn worst_case_byte_offsets(ops: &[AllocatedAbstractOp]) -> Vec<u64> {
    let mut offsets = Vec::with_capacity(ops.len() + 1);
    let mut cur = 0u64;
    offsets.push(cur);
    for op in ops {
        cur += AllocatedAbstractInstructionSet::worst_case_instruction_size(op) * 4;
        offsets.push(cur);
    }
    offsets
}

/// If `ops` already ends with a `LiteralPool` op, return its index and the
/// current serialized byte size of its entries (the offset the next appended
/// entry would receive). Returns `None` if there is no pool yet.
fn pool_state(ops: &[AllocatedAbstractOp]) -> Option<(usize, u64)> {
    let idx = ops
        .iter()
        .rposition(|op| matches!(&op.opcode, Either::Left(AllocatedInstruction::LiteralPool(_))))?;
    if let Either::Left(AllocatedInstruction::LiteralPool(entries)) = &ops[idx].opcode {
        Some((idx, DataSection::literal_pool_size_bytes(entries)))
    } else {
        unreachable!("matched a LiteralPool op but could not borrow its entries")
    }
}

/// Append `entry` to the function's literal pool, returning its index within
/// the pool. If the function has no `LiteralPool` op yet, append one (it becomes
/// the function's last op, sitting after the return — never executed). The
/// pool, once created, is always the last op, so a reverse search finds it and
/// subsequent appends extend it in place.
fn append_entry_to_pool(ops: &mut Vec<AllocatedAbstractOp>, entry: Entry) -> PoolEntryId {
    if let Some(op) = ops
        .iter_mut()
        .rev()
        .find(|op| matches!(&op.opcode, Either::Left(AllocatedInstruction::LiteralPool(_))))
    {
        if let Either::Left(AllocatedInstruction::LiteralPool(entries)) = &mut op.opcode {
            let pool_idx = entries.len() as u32;
            entries.push(entry);
            return PoolEntryId(pool_idx);
        }
        unreachable!("matched a LiteralPool op but could not borrow its entries");
    }

    // No pool yet: create one at the function's end with this single entry.
    ops.push(AllocatedAbstractOp {
        opcode: Either::Left(AllocatedInstruction::LiteralPool(vec![entry])),
        comment: "literal pool".into(),
        owning_span: None,
    });
    PoolEntryId(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asm_generation::fuel::data_section::EntryName;
    use crate::asm_lang::allocated_ops::AllocatedRegister;
    use crate::asm_lang::ConstantRegister;

    /// Build an `AllocatedAbstractInstructionSet` from raw ops (no function
    /// metadata; the pass only inspects `ops`).
    fn func(ops: Vec<AllocatedAbstractOp>) -> AllocatedAbstractInstructionSet {
        AllocatedAbstractInstructionSet {
            function: None,
            ops,
        }
    }

    fn empty_prologue() -> AllocatedAbstractInstructionSet {
        AllocatedAbstractInstructionSet {
            function: None,
            ops: vec![],
        }
    }

    /// A `LoadFromDataSection` op referencing `id` into register `reg`.
    fn load_op(reg: u8, id: &DataId) -> AllocatedAbstractOp {
        AllocatedAbstractOp {
            opcode: Either::Left(AllocatedInstruction::LoadFromDataSection(
                AllocatedRegister::Allocated(reg),
                id.clone(),
            )),
            comment: String::new(),
            owning_span: None,
        }
    }

    /// An `AddrFromDataSection` op referencing `id` into register `reg`.
    fn addr_op(reg: u8, id: &DataId) -> AllocatedAbstractOp {
        AllocatedAbstractOp {
            opcode: Either::Left(AllocatedInstruction::AddrFromDataSection(
                AllocatedRegister::Allocated(reg),
                id.clone(),
            )),
            comment: String::new(),
            owning_span: None,
        }
    }

    /// A trivial fixed-size op (used to pad a function so the pool is far enough
    /// to exercise the reach check).
    fn noop() -> AllocatedAbstractOp {
        AllocatedAbstractOp {
            opcode: Either::Left(AllocatedInstruction::NOOP),
            comment: String::new(),
            owning_span: None,
        }
    }

    /// A `RET` (a non-fall-through op; irrelevant to the pass logic but lets a
    /// function look realistic).
    fn ret_op() -> AllocatedAbstractOp {
        AllocatedAbstractOp {
            opcode: Either::Left(AllocatedInstruction::RET(
                AllocatedRegister::Constant(ConstantRegister::One),
            )),
            comment: String::new(),
            owning_span: None,
        }
    }

    /// Extract the literal pool of `func`, if any (expected as its last op).
    fn pool_entries(func: &AllocatedAbstractInstructionSet) -> Option<&Vec<Entry>> {
        func.ops.last().and_then(|op| match &op.opcode {
            Either::Left(AllocatedInstruction::LiteralPool(entries)) => Some(entries),
            _ => None,
        })
    }

    /// True iff `func` has an `AddrFromLiteralPool` op with the given pool id.
    fn has_addr_from_pool(func: &AllocatedAbstractInstructionSet, pool_id: &PoolEntryId) -> bool {
        func.ops.iter().any(|op| {
            matches!(
                &op.opcode,
                Either::Left(AllocatedInstruction::AddrFromLiteralPool(_, id)) if id == pool_id
            )
        })
    }

    #[test]
    fn single_use_addr_relocated() {
        let mut data = DataSection::default();
        let id = data.insert_data_value(Entry::new_word(
            0xDEAD_BEEF_CAFE_BABE,
            EntryName::NonConfigurable,
            None,
        ));
        let mut f = func(vec![addr_op(0, &id), ret_op()]);

        move_to_literal_pools(&empty_prologue(), std::slice::from_mut(&mut f), &mut data);

        assert!(has_addr_from_pool(&f, &PoolEntryId(0)));
        let entries = pool_entries(&f).expect("function should have a literal pool");
        assert_eq!(entries.len(), 1);
        assert!(entries[0].equiv(&data.non_configurables[0]));
        assert!(data.is_relocated(&id));
    }

    #[test]
    fn multi_use_same_function_relocated_once() {
        let mut data = DataSection::default();
        let id = data.insert_data_value(Entry::new_word(42, EntryName::NonConfigurable, None));
        let mut f = func(vec![addr_op(0, &id), addr_op(1, &id), ret_op()]);

        move_to_literal_pools(&empty_prologue(), std::slice::from_mut(&mut f), &mut data);

        assert_eq!(
            f.ops
                .iter()
                .filter(|op| {
                    matches!(
                        &op.opcode,
                        Either::Left(AllocatedInstruction::AddrFromLiteralPool(_, PoolEntryId(0)))
                    )
                })
                .count(),
            2
        );
        let entries = pool_entries(&f).expect("function should have a literal pool");
        assert_eq!(entries.len(), 1, "entry must be pooled only once");
        assert!(data.is_relocated(&id));
    }

    #[test]
    fn multi_function_use_kept_in_data_section() {
        let mut data = DataSection::default();
        let id = data.insert_data_value(Entry::new_word(7, EntryName::NonConfigurable, None));
        let f0 = func(vec![addr_op(0, &id), ret_op()]);
        let f1 = func(vec![addr_op(0, &id), ret_op()]);
        let mut functions = [f0, f1];

        move_to_literal_pools(&empty_prologue(), &mut functions, &mut data);

        for f in &functions {
            assert!(
                !has_addr_from_pool(f, &PoolEntryId(0)),
                "cross-function use must stay in the data section"
            );
            assert!(pool_entries(f).is_none(), "no pool should be created");
        }
        assert!(!data.is_relocated(&id));
    }

    #[test]
    fn value_load_use_kept_in_data_section() {
        let mut data = DataSection::default();
        let id = data.insert_data_value(Entry::new_word(99, EntryName::NonConfigurable, None));
        let mut f = func(vec![load_op(0, &id), ret_op()]);

        move_to_literal_pools(&empty_prologue(), std::slice::from_mut(&mut f), &mut data);

        assert!(f.ops.iter().any(|op| matches!(
            &op.opcode,
            Either::Left(AllocatedInstruction::LoadFromDataSection(_, _))
        )));
        assert!(pool_entries(&f).is_none());
        assert!(!data.is_relocated(&id));
    }

    #[test]
    fn configurable_never_relocated() {
        let mut data = DataSection::default();
        let id = data.insert_data_value(Entry::new_word(
            5,
            EntryName::Configurable("CFG".into()),
            None,
        ));
        let mut f = func(vec![addr_op(0, &id), ret_op()]);

        move_to_literal_pools(&empty_prologue(), std::slice::from_mut(&mut f), &mut data);

        assert!(
            !has_addr_from_pool(&f, &PoolEntryId(0)),
            "configurables must never be relocated"
        );
        assert!(pool_entries(&f).is_none());
        assert!(!data.is_relocated(&id));
    }

    /// An entry whose only use is too far from the function-end pool (beyond
    /// the 12-bit `ADDI` reach) is left in the data section rather than
    /// producing an out-of-range `ADDI`.
    #[test]
    fn out_of_range_use_kept_in_data_section() {
        let mut data = DataSection::default();
        let id = data.insert_data_value(Entry::new_word(11, EntryName::NonConfigurable, None));

        // Pad with enough fixed-size ops that the pool would sit >4095 bytes
        // past the use. Each NOOP is 4 bytes; 1030 NOOPs = 4120 bytes > 4095.
        let mut ops = vec![addr_op(0, &id)];
        ops.extend(std::iter::repeat_with(noop).take(1030));
        ops.push(ret_op());
        let mut f = func(ops);

        move_to_literal_pools(&empty_prologue(), std::slice::from_mut(&mut f), &mut data);

        assert!(
            !has_addr_from_pool(&f, &PoolEntryId(0)),
            "out-of-range use must stay in the data section"
        );
        assert!(pool_entries(&f).is_none());
        assert!(!data.is_relocated(&id));
    }

    /// An entry used from the prologue (e.g. a selector value-load) must stay in
    /// the data section even if also address-used in one function — otherwise
    /// the prologue's reference would read a tombstoned entry.
    #[test]
    fn prologue_use_kept_in_data_section() {
        let mut data = DataSection::default();
        let id = data.insert_data_value(Entry::new_word(33, EntryName::NonConfigurable, None));
        let prologue = func(vec![load_op(0, &id)]);
        let f = func(vec![addr_op(0, &id), ret_op()]);
        let mut functions = [f];

        move_to_literal_pools(&prologue, &mut functions, &mut data);

        assert!(
            !has_addr_from_pool(&functions[0], &PoolEntryId(0)),
            "an entry referenced from the prologue must stay in the data section"
        );
        assert!(pool_entries(&functions[0]).is_none());
        assert!(!data.is_relocated(&id));
    }
}