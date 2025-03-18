script;

mod lib;
mod lib2; // Contains uses of std::prelude items to ensure they are imported correctly.

// Glob import should shadow the Add trait from the core prelude
use lib::*;

struct S {
    a: u64
}

impl Add for S {
    fn add (self, other: S) -> u64 {
        self.a - other.a
    }
}


fn main() -> u64 {
    let x = S { a : 42 };
    let y = S { a : 64 };

    // lib::log shadows std::logging::log
    let overridden_log_value = log(x);
    
    y.add(x) + overridden_log_value
}
