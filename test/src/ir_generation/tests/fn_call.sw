script;

fn a(x: u64) -> u64 {
    x
}

fn b(x: u64, y: u64) -> u64 {
    let var: bool = false;
    if var {
        x
    } else {
        y
    }
}

fn main() -> u64 {
    a(0);
    a(1);
    b(11, 22)
}

// ::check-ir::

// check: fn main() -> u64
// check:     call
// check:     call
// check:     call
// check:     ret u64

// check: fn $ID(x $MD: u64) -> u64
// check:     entry(x: u64):
// check:     ret u64 $VAL
// check: }

// check: fn $ID(x $MD: u64, y $MD: u64) -> u64
// check:     local bool var

// ::check-asm::
//
// regex: IMM=i\d+
// regex: REG=\$[[:alpha:]][0-9[:alpha:]]*
//
// Matching fn a() here, which just returns its arg:
//
// check: move $$$$retv $REG
// check: jal  $$zero $$$$reta i0
//
// Matching fn b() here, which has a local bool var, initialised to false/$zero:
//
// check: move $$$$locbase $$sp
// check: cfei i24
//
// check: sb   $$$$locbase $$zero i0
// ...
// check: cfsi i24
// check: jal  $$zero $$$$reta i0
