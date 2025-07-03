// This is more to test the ASM generation rather than IR.

script;

fn add(lhs: u64, rhs: u64) -> u64 {
    asm (l: lhs, r: rhs, x) {
        add x r l;
        x: u64
    }
}

fn f(a: u64, b: u64, c: u64) -> u64 {
    add(a, add(b, c))
}

fn g(x: u64, y: u64, z: u64) -> u64 {
    f(f(x, x, y), f(y, y, z), f(z, z, z))
}

fn main() -> u64 {
    g(1, 10, 100)
}

// ::check-ir::

// check: fn main() -> u64

// ::check-asm::

// regex: IMM=i\d+
// regex: REG=\$[[:alpha:]][0-9[:alpha:]]*

// check: move $$$$arg0 $REG
// check: move $$$$arg1 $REG
// check: move $$$$arg2 $REG
// check: sub  $$$$reta $$pc $$is
// check: srli $$$$reta $$$$reta i2
// check: addi $$$$reta $$$$reta i4
// check: jmpf $$zero $IMM
// check: ret  $$$$retv
