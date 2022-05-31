script;

use std::assert::assert;

struct S {
    t: (u64,)
}

fn main() -> u64 {
    let a = S {
        t: (2,)
    };
    let b = match a {
        S { t } => t,
    };
    assert(b.0 == 2);

    1
}
