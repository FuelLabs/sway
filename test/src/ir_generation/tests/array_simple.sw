script;

fn main() -> bool {
    let a = [false, true, false];
    a[1]
}

// ::check-ir::

// check: local [bool; 3] a

// check: store

// check: $(var_val=$VAL) = get_local [bool; 3] a
// check: $(idx_val=$VAL) = const u64 1
// check: $(ret_val=$VAL) = extract_element $var_val, [bool; 3], $idx_val
// check: ret bool $ret_val
