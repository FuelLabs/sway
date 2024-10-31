// This test proves that https://github.com/FuelLabs/sway/issues/6388 is fixed.
script;

struct S {
    x: bool
}

fn main() {
    let v = 0u64;
    // We want to have only one error in the case below,
    // for the whole or-pattern elements and not for individual
    // parts.
    match v {
        1 => (),
        1 | 1 => (),
        _ => (),
    };

    match v {
        1 => (),
        2 => (),
        1 | 2 => (),
        _ => (),
    };

    // TODO: Once https://github.com/FuelLabs/sway/issues/5097 is fixed, the below examples
    //       will also emit warnings for unreacahbility of individual or-pattern elements.
    //       Extend filecheck checks to cover those warning.

    match v {
        1 => (),
        2 => (),
        1 | 3 => (),
        2 | 4 => (),
        _ => (),
    };

    let s = S { x: false };
    let _x = match s {
        S { x } | S { x } => { x },
    };
}
