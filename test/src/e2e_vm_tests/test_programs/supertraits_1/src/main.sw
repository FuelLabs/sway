script;

use std::chain::assert;

/// Traits ///
trait Type {
    fn get_number_of_bytes(self) -> u64;
    fn is_numeric(self) -> bool;
    fn is_boolean(self) -> bool;
} {
    fn get_number_of_bits(self) -> u64 {
        self.get_number_of_bytes() * 8
    }
    fn is_not_numeric(self) -> bool {
        !self.is_numeric()
    }
    fn is_not_boolean(self) -> bool {
        !self.is_boolean()
    }
}

trait Numeric : Type {}

trait Boolean : Type {}

/// Structs ///
struct U64 {
    n: u64,
}

struct Bool {
    b: bool,
}

/// Impl of traits ///
impl Numeric for U64 {
    fn get_number_of_bytes(self) -> u64 {
        8
    }
    fn is_numeric(self) -> bool {
        true
    }
    fn is_boolean(self) -> bool {
        false 
    }
}

impl Boolean for Bool {
    fn get_number_of_bytes(self) -> u64 {
        1
    }
    fn is_numeric(self) -> bool {
        false 
    }
    fn is_boolean(self) -> bool {
        true
    }
}

fn main() -> bool {
    let b = Bool { b: true };
    assert(!b.is_numeric());
    assert(b.is_not_numeric());
    assert(b.is_boolean());
    assert(!b.is_not_boolean());
    assert(b.get_number_of_bytes() == 1);
    assert(b.get_number_of_bits() == 8);

    let n = U64 { n: 55 };
    assert(n.is_numeric());
    assert(!n.is_not_numeric());
    assert(!n.is_boolean());
    assert(n.is_not_boolean());
    assert(n.get_number_of_bytes() == 8);
    assert(n.get_number_of_bits() == 64);

    true
}
