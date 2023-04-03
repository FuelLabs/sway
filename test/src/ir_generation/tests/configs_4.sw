// target-fuelvm

script;

const A: u64 = 42;
const B: u64 = 42;

configurable {
    X: u64 = 11,
    Y: u64 = 11,
}

fn main() -> (u64, u64, u64, u64) {
    (A, B, X, Y)
}

// ::check-ir::

// check: script

// ::check-asm::

// regex: DATA=data_\d+
// regex: REG=\$r\d+

// There should only be 3 data entries here. One shared by `A` and `B` and the
// other two are for `X` and `Y` respectively

// unordered: $DATA .word 42
// unordered: $DATA .word 11
// unordered: $DATA .word 11
