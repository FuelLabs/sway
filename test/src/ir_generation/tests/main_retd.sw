// target-fuelvm

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

// check: fn main() -> __ptr { u64, ( { b256, b256 } | { u64, { u64, ( b256 | u64 ) } } ) }
// check: ret __ptr { u64, ( { b256, b256 } | { u64, { u64, ( b256 | u64 ) } } ) }

// ::check-asm::

// regex: REG=\$r\d+
// regex: ID=[_[:alpha:]][_0-9[:alpha:]]*

// B is 48 bytes.
// check: mcpi $REG $REG i48

// D is 72 bytes.
// check: movi $(len_reg=$REG) i72
// check: retd  $$$$locbase $len_reg
