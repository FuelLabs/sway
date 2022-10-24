script;

enum A {
    Z: b256,
    Y: u64,
}

struct B {
    x: u64,
    w: A,
}

struct C {
    v: b256,
    u: b256,
}

enum D {
    T: C,
    S: B,
}

fn main() -> D {
    D::S(B { x: 33, w: A::Y(44) })
}

// ::check-ir::

// check: fn main() -> { u64, ( { b256, b256 } | { u64, { u64, ( b256 | u64 ) } } ) }
// check: ret { u64, ( { b256, b256 } | { u64, { u64, ( b256 | u64 ) } } ) }

// ::check-asm::

// regex: REG=\$r\d+
// regex: ID=[_[:alpha:]][_0-9[:alpha:]]*

// B is 48 bytes.
// check: mcpi $REG $REG i48

// D is 72 bytes.
// check: lw   $(len_reg=$REG) $(len_data=$ID)
// check: retd  $REG $len_reg

// check: .data:
// check: $len_data .word 72
