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

// unordered: $(v1=$VAL) = insert_value $VAL, { b256, u64 }, $c1, 0
// unordered: $(v2=$VAL) = insert_value $v1, { b256, u64 }, $c0, 1
// check: ret { b256, u64 } $v2

// unordered: $(config_name_X=$MD) = config_name "X"
// unordered: $config0_mds = ($MD $config_name_X)

// unordered: $(config_name_Y=$MD) = config_name "Y"
// unordered: $config1_mds = ($MD $config_name_Y)

// ::check-asm::

// regex: IMM=i\d+
// regex: REG=\$r\d+
