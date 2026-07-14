script;

// A mostly-zeroed tuple: seven zero elements and a single non-zero one, i.e. a
// zero-ratio of 7/8 = 0.875. Such a high ratio is well above the mostly-zeroed
// threshold, so the optimization triggers regardless of the exact threshold
// value. The tuple is lowered to a single `mem_clear_val` for the whole tuple,
// followed by a single `store` of the one non-zero element. None of the zero
// elements are stored individually.

fn main() -> u64 {
    let t = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 42u64);
    t.7
}

// ::check-ir::
// check: local { u64, u64, u64, u64, u64, u64, u64, u64 } __tuple_init_0
// check: init_aggr

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(tuple_ptr=$VAL) = get_local __ptr { u64, u64, u64, u64, u64, u64, u64, u64 }, __tuple_init_0

// The whole tuple is zero-cleared first.
// check: mem_clear_val $tuple_ptr

// Only the single non-zero element is stored; no zero element is stored.
// check: $(idx_7=$VAL) = const u64 7
// check: $(elem_ptr=$VAL) = get_elem_ptr $tuple_ptr, __ptr u64, $idx_7
// check: $(c_42=$VAL) = const u64 42
// check: store $c_42 to $elem_ptr

// nextln: $VAL = load $tuple_ptr

// There must be no `init_aggr` left after lowering.
// not: init_aggr
