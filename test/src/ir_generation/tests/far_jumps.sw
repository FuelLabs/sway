script;

fn main() -> bool {
    a() && b() && b()
}

fn a() -> bool {
    asm (res) {
        // Introduce a 'NOP blob' to push the address of b() out into the danger zone.
        blob i262200;
        movi res i1;
        res: bool
    }
}

fn t() -> bool {
    asm() {
        one: bool
    }
}

fn b() -> bool {
    // Create complex control flow.
    while t() {
        t() && t();
    }
    t()
}

// ::check-ir::

// check: fn main() -> bool
// check: call a$()
// check: call b$()
// check: call b$()

// The blob must be before b().
// check: blob

// We want both `cbr`s and `br`s in b().
// check: fn b$()
// check: call t$()
// check: cbr
// check: br

// ::check-asm::

// regex: REG=\$r\d+

// This test is solved via code relocation.  So a() and the blob should be moved to beyond the
// control flow and keep the rest of b() in the sub 1MB space.

// Some conditional flow.
// check: jnzi

// The guts of a() with the blob.
// check: blob i262200
// check: movi $(ret_reg=$REG) i1
// check: move $$$$retv $ret_reg

// Now we don't want to see any conditional control flow.
// not: jnzi
// not: jnei
