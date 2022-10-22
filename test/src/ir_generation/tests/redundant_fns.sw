// This is to test that we don't recreate the same function for every time it's called any more.

script;

// -------------------------------------------------------------------------------------------------
// `a` and `b` are called multiple times but should only exist once each in the IR.

fn a(x: bool) -> bool {
    x || x
}

fn b(y: bool) -> bool {
    a(a(y))
}

fn c(z: bool) -> bool {
    b(z) && b(z)
}

// -------------------------------------------------------------------------------------------------
// `Id::id()` for `u64` and `bool` are different functions and should exist separately.

trait Id {
    fn id(self) -> Self;
}

impl Id for u64 {
    fn id(self) -> Self {
        self
    }
}

impl Id for bool {
    fn id(self) -> Self {
        self
    }
}

// -------------------------------------------------------------------------------------------------

fn main() -> bool {
    11u64.id();
    false.id();

    c(true)
}

// The names are still uniqued with a `_x` suffix though.
//
// regex: A_FN=a_\d
// regex: B_FN=b_\d
// regex: C_FN=c_\d

// check: fn main

// check: $(=^\s*)pub fn id_
// sameln: u64
// check: $(=^\s*)pub fn id_
// sameln: bool

// check: fn $C_FN
// check: fn $B_FN
// check: fn $A_FN

// not: fn $A_FN
// not: fn $B_FN
// not: fn $C_FN
