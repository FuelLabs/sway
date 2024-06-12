script;

enum Enum {
    A: (u64),
    B: (u64),
}

struct Struct {
    x: u64,
    y: u64,
    z: u64
}
 
// For testing side effects.
fn inc_i(ref mut i: u64) -> Struct {
    i = i + 11;
 
    Struct { x: 21, y: 21, z: 1 }
}

fn main() {
    let x = match 8 {
        7 => { 4 },
        9 => { 5 },
        8 => { 42 },
        _ => { 100 },
    };
    assert(x == 42);

    let a = 5;
    let x = match a {
        7 => { 4 },
        5 => { 42 },
        _ => { 24 },
    };
    assert(x == 42);

    let a = 5;
    let x = match a {
        7 | 8 | 9 => { 4 },
        3 | 4 | 5 => { 42 },
        _ => { 24 },
    };
    assert(x == 42);

    // Test side effects. `inc_i` must be called exactly once.
    let mut i = 0;
    let x = match inc_i(i) {
        Struct { x, y, z: 0 } => x + y,
        Struct { x, y, z: 1 } => x + y,
        _ => 24,
    };
    assert(i == 11);
    assert(x == 42);

    // Test match expressions with just one arm.
    let e = Enum::A(42);

    let x = match e {
        _ => 9999,
    };
    assert(x == 9999);

    let e = Enum::B(42);
    let x = match e {
        Enum::A(x) | Enum::B(x) => x,
    };
    assert(x == 42);

    let x = match e {
        Enum::A(_) | Enum::B(_) => 9999,
    };
    assert(x == 9999);

    let e = 42u64;
    let x = match e {
        y => y,
    };
    assert(x == 42);

    let mut i = 0;
    match e {
        _ => {
            let _s = inc_i(i);
        }
    };
    assert(i == 11);

    let r = match 42 {
        0 => { 24 },
        foo => { foo },
    };
    assert(r == 42);

    // string slice

    let r = match "b" {
        "get_a" => { 1 },
        "get_b" => { 2 },
        _ => { 3 },
    };
    assert(r == 2);
}
