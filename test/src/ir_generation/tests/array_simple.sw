script;

fn main() -> bool {
    let a = [false, true, false];
    a[1]
}

// ::check-ir::

// check: local [bool; 3] __array_init_0
// check: local [bool; 3] a

// check: $(ptr_array_init=$VAL) = get_local __ptr [bool; 3], __array_init_0
// check: $(c_false_1=$VAL) = const bool false
// check: $(c_true_2=$VAL) = const bool true
// check: $(c_false_3=$VAL) = const bool false
// check: $(ptr_init_aggr=$VAL) = init_aggr $ptr_array_init [$c_false_1, $c_true_2, $c_false_3]
// check: $(loaded_init_aggr=$VAL) = load $ptr_init_aggr
// check: $(ptr_a=$VAL) = get_local __ptr [bool; 3], a
// check: store $loaded_init_aggr to $ptr_a
// check: $(ptr_a=$VAL) = get_local __ptr [bool; 3], a
// check: $(idx_1=$VAL) = const u64 1
// check: $(ptr_elem=$VAL) = get_elem_ptr $ptr_a, __ptr bool, $idx_1
// check: $(loaded_elem=$VAL) = load $ptr_elem
// check: ret bool $loaded_elem

// ::check-ir-optimized::
// pass: lower-init-aggr

// check: $(ptr_array_init=$VAL) = get_local __ptr [bool; 3], __array_init_0
// check: mem_clear_val $ptr_array_init,
// check: $(id_1=$VAL) = const u64 1
// check: $(ptr_array_1=$VAL) = get_elem_ptr $ptr_array_init, __ptr bool, $id_1
// check: $(c_true_0=$VAL) = const bool true
// check: store $c_true_0 to $ptr_array_1
// check: $(load_array_init=$VAL) = load $ptr_array_init
// check: $(ptr_a=$VAL) = get_local __ptr [bool; 3], a
// check: store $load_array_init to $ptr_a
// check: $(ptr_a=$VAL) = get_local __ptr [bool; 3], a
// check: $(id_1=$VAL) = const u64 1
// check: $(ptr_array_1=$VAL) = get_elem_ptr $ptr_a, __ptr bool, $id_1
// check: $(loaded_elem=$VAL) = load $ptr_array_1
// check: ret bool $loaded_elem
