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

// check: fn main($ID: ptr { u64, ( { b256, b256 } | { u64, { u64, ( b256 | u64 ) } } ) }) -> ptr { u64, ( { b256, b256 } | { u64, { u64, ( b256 | u64 ) } } ) }
// check: ret ptr { u64, ( { b256, b256 } | { u64, { u64, ( b256 | u64 ) } } ) }

// ::check-asm::

// regex: REG=\$r\d+
// regex: ID=[_[:alpha:]][_0-9[:alpha:]]*

// B is 48 bytes.
// check: movi $(len_reg=$REG) i48
// check: mcp  $REG $REG $len_reg

// D is 72 bytes.
// check: movi $(len_reg=$REG) i72
// check: mcp  $(ptr_reg=$REG) $REG $len_reg
// check: lw   $(len_reg=$REG) $(len_data=$ID)
// check: retd  $ptr_reg $len_reg

// check: .data:
// check: $len_data .word 72
