library;

struct A {
    a: u64,
    b: u64,
}

struct B {
    a: A,
    b: u64,
}

pub fn or_patterns_test() {
    match 0 {
        1 | 2 => (),
        0 => (),
    }

    let a = A { a: 1, b: 2 };


    match a {
        A { a, b: 2 } => (),
        A { a, b: 1 } => (),
        A  { a: 1, b: _ } => (),
    }

    match a {
        A { a, b: 2 } | A { a, b: 1 } => (),
        A  { a: 1, b: 0 } => (),
        A  { a: 2, b: _ } => (),
        A { a, b: 3 } => (),
    }

    let b = B { a, b: 1 };

    match b {
        B {
            a: A { a, b: 2 } | A { a, b: 1 },
            b: _,
        } => (),
        B {
            a: _,
            b: 1,
        } => (),
    }

    match b {
        B {
            a: A { a, b: 2 } ,
            b: _,
        } | B {
            a: A { a, b: 1 },
            b: _,
        } => (),
        B {
            a: _,
            b: 1,
        } => (),
    }
}
