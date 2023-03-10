script;

mod lib;
use lib::{B, C, D,};

fn main() -> u64 {
    let x = B {
        b: 0,
    };
    let y = C {
        c: 0,
    };
    let z = D {
        d: 0,
    };
    let foo = x.b + y.c + z.d;
    foo
}
