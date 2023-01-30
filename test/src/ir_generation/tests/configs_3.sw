// target-fuelvm

script;

configurable {
    X: u64 = 42,
    Y: u64 = 42,
}

fn main() -> (u64, u64) {
    (X, Y)
}

// ::check-ir::

// regex: CONFIG=c\d+

// check: script {
// check: $(c0=$CONFIG) = config u64 42, $(config0_mds=$MD)
// check: $(c1=$CONFIG) = config u64 42, $(config1_mds=$MD)

// check: $(temp_ptr=$VAL) = get_local ptr { u64, u64 }, $ID

// check: $(idx_val=$VAL) = const u64 0
// check: $(field_ptr=$VAL) = get_elem_ptr $temp_ptr, ptr u64, $idx_val
// check: store c0 to $field_ptr

// check: $(idx_val=$VAL) = const u64 1
// check: $(field_ptr=$VAL) = get_elem_ptr $temp_ptr, ptr u64, $idx_val
// check: store c1 to $field_ptr


// unordered: $(config_name_X=$MD) = config_name "X"
// unordered: $config0_mds = ($MD $config_name_X)

// unordered: $(config_name_Y=$MD) = config_name "Y"
// unordered: $config1_mds = ($MD $config_name_Y)

// ::check-asm::

// regex: DATA=data_\d+

// Make sure there are two data locations, one for each config variable
// check: $DATA .word 42
// check: $DATA .word 42
