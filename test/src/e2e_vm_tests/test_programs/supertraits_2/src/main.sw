script;

use std::chain::assert;

/// Traits ///
trait Numeric {
    fn is_numeric(self) -> bool;
} {
    fn is_not_numeric(self) -> bool {
        !self.is_numeric()
    }
}

trait Boolean {
    fn is_boolean(self) -> bool;
} {
    fn is_not_boolean (self) -> bool {
        !self.is_boolean()
    }
}

trait NumericAndBoolean : Numeric + Boolean {
    fn is_struct(self) -> bool;
} {
fn is_not_struct (self) -> bool {
        !self.is_struct()
    }
}

/// Structs ///
struct U64 {
    n: u64,
}

struct Bool {
    b: bool,
}

struct U64AndBool {
    n: u64,
    b: bool,
}

/// Impl of traits ///
impl Numeric for U64 {
    fn is_numeric(self) -> bool {
        true
    }
}

impl Boolean for Bool {
    fn is_boolean(self) -> bool {
        true
    }
}

impl NumericAndBoolean for U64AndBool {
    fn is_numeric(self) -> bool {
        false 
    }
    fn is_boolean(self) -> bool {
        false 
    }
    fn is_struct(self) -> bool {
        true 
    }
}

fn main() -> bool {
    let b = Bool { b: true };
    assert(b.is_boolean());
    assert(!b.is_not_boolean());

    let n = U64 { n: 55 };
    assert(n.is_numeric());
    assert(!n.is_not_numeric());

    let nb = U64AndBool { n: 7, b: false };
    assert(!nb.is_numeric());
    assert(nb.is_not_numeric());
    assert(!nb.is_boolean());
    assert(nb.is_not_boolean());
    assert(nb.is_struct());
    assert(!nb.is_not_struct());

    true
}
