script;

fn main() -> bool {
    let a = [false, true, false];
    a[1]
}

// ::check-ir::

// check: local ptr [bool; 3] a

// check: store

// check: $(ptr_val=$VAL) = get_ptr ptr [bool; 3] a, ptr [bool; 3], 0
// check: $(idx_val=$VAL) = const u64 1
// check: $(ret_val=$VAL) = extract_element $ptr_val, [bool; 3], $idx_val
// check: ret bool $ret_val
