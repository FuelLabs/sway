script;

fn main() -> bool {
    let a = [false, true, false];
    a[1]
}

// ::check-ir::

// check: local [bool; 3] a

// check: store

// check: $VAL = get_local __ptr [bool; 3], a

// check: $(var_val=$VAL) = get_local __ptr [bool; 3], a
// check: $(idx_val=$VAL) = const u64 1
// check: $(ptr_val=$VAL) = get_elem_ptr v12, __ptr bool
// check: $(ret_val=$VAL) = load $ptr_val
// check: ret bool $ret_val
