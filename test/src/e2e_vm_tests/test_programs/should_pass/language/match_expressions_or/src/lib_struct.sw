library;

struct A {
    a: u64,
    b: u64,
}

fn match_struct_a(a: A) -> u64 {
    match a {
        A { a, b: 121 } | A { a, b: 122 } => a + a,
        A { a: a, b: 221 } | A { a, b: 222 } | A { a: 213, b: a }  => a * a,
        A { a, b } => {
            return a - b;
        }
    }
}

struct B {
    x: u64,
    y: u64,
    z: u64,
}

fn match_struct_b(b: B) -> u64 {
    match b {
        B { x: 111, y: 122, z: 133 } | B { x: 211, y: 222, z: 233 } | B { x: 311, y: 322, z: 333 } => 5555,
        B { x, y: 222, z: 333 } | B { x: 101, y: x, z: 303 } | B { x: 101, y: 202, z: x } => x,
        B { x: a, y: b, z: c} | B { x: b, y: c, z: a} => a - b - c,
    }
}

struct EmptyStruct { }

struct EmptyStructContainer {
    e: EmptyStruct,
}

fn match_empty_struct(c: EmptyStructContainer) -> u64 {
    match c {
        EmptyStructContainer { e: EmptyStruct { } } => 42,
    }
}

struct Struct { 
    x: u64,
    y: u64,
    z: u64,
}

// Checks that the reported bug is fixed: https://github.com/FuelLabs/sway/issues/5122
fn test_bug_fix() -> u64 {
    let s = Struct { x: 1, y: 2, z: 3, };

    let a = match s {
        Struct { x, y: 2, z: 3 } | Struct { x: 0, y: x, z: 0 } | Struct { x: 0, y: 0, z: x } => x,
        _ => 1111,
    };
    
    assert(a == 1);

    let a = match s {
        Struct { x, y: 0, z: 0 } | Struct { x: 1, y: x, z: 3 } | Struct { x: 0, y: 0, z: x } => x,
        _ => 1111,
    };
    
    assert(a == 2);

    let a = match s {
        Struct { x, y: 0, z: 0 } | Struct { x: 0, y: x, z: 0 } | Struct { x: 1, y: 2, z: x } => x,
        _ => 1111,
    };

    assert(a == 3);

    let a = match s {
        Struct { x, y: 0, z: 0 } | Struct { x: 0, y: x, z: 0 } | Struct { x: 0, y: 0, z: x } => x,
        _ => 1111,
    };
    
    assert(a == 1111);

    42
}

pub fn test() -> u64 {
    let x = match_struct_a(A { a: 21, b: 121 });
    assert(x == 42);

    let x = match_struct_a(A { a: 21, b: 122 });
    assert(x == 42);

    let x = match_struct_a(A { a: 12, b: 221 });
    assert(x == 144);

    let x = match_struct_a(A { a: 12, b: 222 });
    assert(x == 144);

    let x = match_struct_a(A { a: 999, b: 900 });
    assert(x == 99);

    let x = match_struct_b(B { x: 111, y: 122, z: 133 });
    assert(x == 5555);

    let x = match_struct_b(B { x: 211, y: 222, z: 233 });
    assert(x == 5555);

    let x = match_struct_b(B { x: 311, y: 322, z: 333 });
    assert(x == 5555);

    let x = match_struct_b(B { x: 42, y: 222, z: 333 });
    assert(x == 42);

    let x = match_struct_b(B { x: 101, y: 42, z: 303 });
    assert(x == 42);

    let x = match_struct_b(B { x: 101, y: 202, z: 42 });
    assert(x == 42);

    let x = match_struct_b(B { x: 342, y: 200, z: 100 });
    assert(x == 42);

    let x = match_empty_struct(EmptyStructContainer { e: EmptyStruct { } });
    assert(x == 42);

    let x = test_bug_fix();
    assert(x == 42);

    poke(EmptyStructContainer { e: EmptyStruct { } }.e);

    42
}

fn poke<T>(_x: T) { }