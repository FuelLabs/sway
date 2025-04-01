script;

mod lib;

use ::lib::LIB_X;
use ::lib::LIB_X as LIB_X_ALIAS;

const LOCAL_X: u64 = 123;

struct S {
    x: u8,
}

impl S {
    const X: u64 = 0;
}

struct S2 {
    s: S,
}

fn function(f_x: u8) {
    let _ = &mut f_x;
}

fn main() {
    let _ = &mut LIB_X;

    let _ = &mut LIB_X_ALIAS;

    let _ = &mut S::X;

    let _ = &mut LOCAL_X;

    let _ = &mut { LOCAL_X }; // No error here.

    // ------------------------

    let a = 123;

    let _ = &mut a;

    let _ = &mut { a }; // No error here.

    let S { x } = S { x: 0 };

    let _ = &mut x;

    let S { x: x } = S { x: 0 };

    let _ = &mut x;

    let S { x: x_1 } = S { x: 0 };

    let _ = &mut x_1;

    let s = S { x: 0 };
    let _ = match s {
        S { x } => {
            let _ = &mut x;
        },
        S { x: x } => {
            let _ = &mut x;
        },
        S { x: x_1 } => {
            let _ = &mut x_1;
        },
    };

    if let S { x } = s {
        let _ = &mut x;
    }

    let vec = Vec::<u64>::new();
    for n in vec.iter() {
        let _ = &mut n;
    }

    function(0);

    // aggregates checking

    let _ = &mut s.x;

    let s2 = S2 { s: S { x: 0 } };

    let _ = &mut s2.s.x;

    let t : (u32, u32) = (0, 0);
    let _ : &mut u32 = &mut t.0;

    let _ = &mut { t }; // No error here.
}

