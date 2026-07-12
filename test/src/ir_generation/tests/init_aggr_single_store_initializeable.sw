script;

// Single-store-initializeable aggregates (here a single-field struct wrapping a
// `u64`) must NOT be lowered via `mem_clear_val`, even when the value is zero.
// Instead they are always initialized with a single `store`.

fn main() -> u64 {
    let s = SingleField { field: 0u64 };
    s.field
}

struct SingleField {
    field: u64,
}

// ::check-ir::
// check: init_aggr

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(struct_ptr=$VAL) = get_local __ptr { u64 }, __struct_init_0

// The single field is initialized with a single `store` of the value.
// check: $(idx_0=$VAL) = const u64 0
// check: $(field_ptr=$VAL) = get_elem_ptr $struct_ptr, __ptr u64, $idx_0
// check: $(c_0=$VAL) = const u64 0
// check: store $c_0 to $field_ptr

// nextln: $VAL = load $struct_ptr

// The aggregate is never zero-cleared, and no `init_aggr` remains.
// not: mem_clear_val
// not: init_aggr
