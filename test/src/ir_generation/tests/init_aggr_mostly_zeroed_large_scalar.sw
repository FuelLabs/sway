script;

// A mostly-zeroed tuple whose single non-zero element is a large scalar (`u256`),
// zero-ratio 7/8 = 0.875. The whole tuple is `mem_clear_val`ed and only the one
// non-zero large scalar is written afterwards.

fn main() -> u256 {
    let t = (0u256, 0u256, 0u256, 0u256, 0u256, 0u256, 0u256, 42u256);
    t.7
}

// ::check-ir::
// check: init_aggr

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(tuple_ptr=$VAL) = get_local __ptr { u256, u256, u256, u256, u256, u256, u256, u256 }, __tuple_init_0

// The whole tuple is zero-cleared first.
// check: mem_clear_val $tuple_ptr

// Only the single non-zero large scalar is written.
// check: $(idx_7=$VAL) = const u64 7
// check: $(elem_ptr=$VAL) = get_elem_ptr $tuple_ptr, __ptr u256, $idx_7
// check: $(c_42=$VAL) = const u256 0x000000000000000000000000000000000000000000000000000000000000002a
// check: store $c_42 to $elem_ptr

// nextln: $VAL = load $tuple_ptr

// There must be no `init_aggr` left after lowering.
// not: init_aggr
