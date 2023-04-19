script;

struct A {
    a: u64,
    b: u64,
}

fn main() -> u64 {
    assert(
        match 0 {
            0 | 1 => 0,
            _ => 1,
        } == 0
    );
    assert(
        match 1 {
            0 | 1 => 0,
            _ => 1,
        } == 0
    );

    let a = A { a: 1, b: 2 };
    assert(
        match a {
            A { a, b: 2 } | A { a, b: 1 } => 0,
            _ => 1,
        } == 0
    );

    let a = A { a: 1, b: 2 };
    assert(
        match a {
            A { a, b: 2 } | A { a, b: 1 } => a,
            _ => 0,
        } == 1
    );

    let a = A { a: 1, b: 3 };
    assert(
        match a {
            A { a, b: 2 } | A { a, b: 1 } => 0,
            _ => 1,
        } == 1
    );

    let a = A { a: 42, b: 3 };
    assert(
        match a {
            A { a, b: 2 } | A { a, b: 3 } | A { a, b: 1 }  => a,
            _ => 1,
        } == 42
    );

    0
}
