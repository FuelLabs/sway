script;

// Check that code uses declared dependency name `std_alt` not package name `std`.
use std_alt::chain::assert;

fn main() -> u64 {
    assert(true);
    0
}
