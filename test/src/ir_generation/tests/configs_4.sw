// target-fuelvm

script;

const A: u64 = 42;
const B: u64 = 42;

configurable {
    X: u64 = 11,
    Y: u64 = 11,
    Z: u64 = 12,
}

fn main() -> (u64, u64, u64, u64, u64) {
    (A, B, X, Y, Z)
}

// ::check-ir::

// check: script

// ::check-asm::

// regex: DATA=data_\d+
// regex: REG=\$r\d+

// There should only be 3 entries here, for `X`, 'Y' and `Z` respectively

// unordered: $DATA .word 11
// unordered: $DATA .word 11
// unordered: $DATA .word 12
