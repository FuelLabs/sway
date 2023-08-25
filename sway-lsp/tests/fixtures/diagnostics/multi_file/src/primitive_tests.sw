library;

pub fn simple_numbers_test() {
    let x = 0;
    // should fail
    let y = match x {
        0 => { 0 },
        10 => { 0 },
        5 => { 0 },
        10 => { 0 },
    };
    // should succeed
    let y = match x {
        0 => { 0 },
        1 => { 0 },
        _ => { 0 },
    };
    // should succeed
    let y = match x {
        0 => { 0 },
        1 => { 0 },
        a => { a },
    };
}

pub fn simple_tuples_test() {
    let x = (1, 2);
    // should fail
    let y = match x {
        (0, 0) => { 0 },
        (1, 1) => { 0 },
        (1, 1) => { 0 },
        (1, 2) => { 0 },
    };
    // should succeed
    let y = match x {
        (0, 0) => { 0 },
        (1, 1) => { 0 },
        _ => { 0 },
    };
    // should succeed
    let y = match x {
        (0, 0) => { 0 },
        (1, 1) => { 0 },
        a => { 0 },
    };
    // should succeed
    let y = match x {
        (0, 0) => { 0 },
        (1, 1) => { 0 },
        (a, b) => { 0 },
    };
}

pub fn variable_not_found_test() {
    // should fail
    let foo = match 42 {
        0 => { newvariable},
        foo => { foo },
    };
}
