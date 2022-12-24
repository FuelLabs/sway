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
// unordered: $(c0=$CONFIG) = config u64 42, $(config0_mds=$MD)
// unordered: $(c1=$CONFIG) = config b256 0x0101010101010101010101010101010101010101010101010101010101010101, $(config1_mds=$MD)

// check: $(v1=$VAL) = insert_value $VAL, { b256, u64 }, $c1, 0
// check: $(v2=$VAL) = insert_value $v1, { b256, u64 }, $c0, 1
// check: ret { b256, u64 } $v2

// unordered: $(config_name=$MD) = config_name "X"
// unordered: $config0_mds = ($MD $config_name)

// unordered: $(config_name=$MD) = config_name "Y"
// unordered: $config1_mds = ($MD $config_name)

// ::check-asm::

// regex: IMM=i\d+
// regex: REG=\$r\d+
