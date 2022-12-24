script;

configurable {
    X: u64 = 42,
}

fn main() -> u64 {
    X
}

// ::check-ir::

// regex: CONFIG=c\d+

// check: script {
// check: $(c0=$CONFIG) = config u64 42, $(config_mds=$MD)

// check: ret u64 $c0
// check: $(config_name=$MD) = config_name "X"
// check: $config_mds = ($MD $config_name)

// ::check-asm::

// regex: IMM=i\d+
// regex: REG=\$r\d+
