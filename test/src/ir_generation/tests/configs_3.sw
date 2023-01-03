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

// unordered: $(v1=$VAL) = insert_value $VAL, { u64, u64 }, $c0, 0
// unordered: $(v2=$VAL) = insert_value $v1, { u64, u64 }, $c1, 1
// check: ret { u64, u64 } $v2

// unordered: $(config_name_X=$MD) = config_name "X"
// unordered: $config0_mds = ($MD $config_name_X)

// unordered: $(config_name_Y=$MD) = config_name "Y"
// unordered: $config1_mds = ($MD $config_name_Y)

// ::check-asm::

// regex: DATA=data_\d+

// Make sure there are two data locations, one for each config variable
// check: $DATA .word 42
// check: $DATA .word 42
