script;

configurable {
    Y: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101,
    X: u64 = 42,
}

fn main() -> (b256, u64) {
    (Y, X)
}

// ::check-ir::

// regex: CONFIG=c\d+

// check: script {
// check: $(c0=$CONFIG) = config u64 42, $(config0_mds=$MD)
// check: $(c1=$CONFIG) = config b256 0x0101010101010101010101010101010101010101010101010101010101010101, $(config1_mds=$MD)

// check: $(tuple_ptr=$VAL) = get_local ptr { b256, u64 }, $ID

// check: $(zero_idx=$VAL) = const u64 0
// check: $(tuple_val=$VAL) = get_elem_ptr $tuple_ptr, ptr b256, $zero_idx
// check: store $c1 to $tuple_val

// check: $(one_idx=$VAL) = const u64 1
// check: $(tuple_val=$VAL) = get_elem_ptr $tuple_ptr, ptr u64, $one_idx
// check: store $c0 to $tuple_val

// check: $(ret_val=$VAL) = load $tuple_ptr
// check: ret { b256, u64 } $ret_val

// unordered: $(config_name_X=$MD) = config_name "X"
// unordered: $config0_mds = ($MD $config_name_X)

// unordered: $(config_name_Y=$MD) = config_name "Y"
// unordered: $config1_mds = ($MD $config_name_Y)
