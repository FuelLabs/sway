script;

const A: u64 = 42;
const B: u64 = 42;

configurable {
    X: u64 = 42,
    Y: u64 = 42,
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

// check: $(d1=$DATA) .word 42
// check: $(d2=$DATA) .word 42
// check: $(d3=$DATA) .word 42
