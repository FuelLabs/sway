script;

fn main() -> u64 {
    let arr = [50u64, 0u64, 0u64, 0u64];
    arr[0]
}

// ::check-ir::
// check: local [u64; 4] __array_init_0

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: local [u64; 4] __array_init_0
// check: $(v1v1=$VAL) = get_local __ptr [u64; 4], __array_init_0,
// check: mem_clear_val $v1v1
// check: $(v16v1=$VAL) = const u64 0
// check: $(v17v1=$VAL) = get_elem_ptr $v1v1, __ptr u64, $v16v1
// check-not: const u64 0
// check-not: $v16v1
// check: $(v2v1=$VAL) = const u64 50, !5
// check: store $v2v1 to $v17v1, !4