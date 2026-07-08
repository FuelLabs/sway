script;

// Small repeat arrays (length <= 5) are lowered into individual `store`s of the
// repeated value into each element. The repeated value is emitted once and
// reused for every element.

fn main() -> u64 {
    let a = [7u64; 3];
    a[1]
}

// ::check-ir::

// check: local [u64; 3] __array_init_0
// check: local [u64; 3] a

// check: $(ptr_array_init=$VAL) = get_local __ptr [u64; 3], __array_init_0
// check: $(c_7=$VAL) = const u64 7
// check: $(ptr_init_aggr=$VAL) = init_aggr $ptr_array_init [$c_7 x 3]
// check: $(loaded_init_aggr=$VAL) = load $ptr_init_aggr
// check: $(ptr_a=$VAL) = get_local __ptr [u64; 3], a
// check: store $loaded_init_aggr to $ptr_a

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(ptr_array_init=$VAL) = get_local __ptr [u64; 3], __array_init_0

// The repeated value is stored into every element individually. The repeated
// value constant is emitted once and reused for all stores.
// check: $(idx_0=$VAL) = const u64 0
// check: $(ptr_array_0=$VAL) = get_elem_ptr $ptr_array_init, __ptr u64, $idx_0
// check: $(c_7=$VAL) = const u64 7
// check: store $c_7 to $ptr_array_0
// check: $(idx_1=$VAL) = const u64 1
// check: $(ptr_array_1=$VAL) = get_elem_ptr $ptr_array_init, __ptr u64, $idx_1
// check: store $c_7 to $ptr_array_1
// check: $(idx_2=$VAL) = const u64 2
// check: $(ptr_array_2=$VAL) = get_elem_ptr $ptr_array_init, __ptr u64, $idx_2
// check: store $c_7 to $ptr_array_2

// check: $(load_array_init=$VAL) = load $ptr_array_init
// check: $(ptr_a=$VAL) = get_local __ptr [u64; 3], a
// check: store $load_array_init to $ptr_a

// There must be no `init_aggr` left after lowering.
// not: init_aggr
