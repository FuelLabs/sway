script;

// A mostly-zeroed, non-repeat array with a single non-zero element at a
// non-leading position (zero-ratio 7/8 = 0.875, well above the threshold). The
// array is `mem_clear_val`ed and only the one non-zero element is stored.

fn main() -> u64 {
    let a = [0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 42u64];
    a[7]
}

// ::check-ir::
// check: local [u64; 8] __array_init_0
// check: init_aggr

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(array_ptr=$VAL) = get_local __ptr [u64; 8], __array_init_0

// The whole array is zero-cleared first.
// check: mem_clear_val $array_ptr

// Only the single non-zero element is stored; no zero element is stored.
// check: $(idx_7=$VAL) = const u64 7
// check: $(elem_ptr=$VAL) = get_elem_ptr $array_ptr, __ptr u64, $idx_7
// check: $(c_42=$VAL) = const u64 42
// check: store $c_42 to $elem_ptr

// nextln: $VAL = load $array_ptr

// There must be no `init_aggr` left after lowering.
// not: init_aggr
