library;

enum Enum {
    A: (u64),
    B: (u64, u64),
    C: (u64, u64, u64),
}

struct StructA {
    x: u64,
    y: u64,
    z: u64,
    e: Enum,
}

struct StructB {
    a: StructA,
    x: u64,
    y: u64,
}

pub fn match_nested(b: StructB) -> u64 {
    match b {
        StructB {
            a: StructA { x: a_x, y: a_y, z: 301, e: _ }
                | StructA { x: 102, y: a_x, z: a_y, e: _ } 
                | StructA { x: 103, y: 203, z: a_x | a_x | a_x, e: Enum::A(a_y) | Enum::B((_, a_y)) | Enum::C((_, _, a_y)) },
            x: b_x,
            y: 111,
        }
        | 
        StructB {
            a: StructA { x: b_x, y: 201, z: 301, e: Enum::A(a_y) | Enum::B((a_y, _)) | Enum::C((a_y, _, _)) }
                | StructA { x: 102, y: b_x, z: a_y, e: _ } 
                | StructA { x: a_y, y: 203, z: b_x, e: _ },
            x: 111,
            y: a_x | a_x | a_x,
        } => a_x + b_x + a_y,
        _ => 9999,
    }
}

pub fn test() -> u64 {
    // First OR variant.

    // a_x: 10, a_y: 20, b_x: 30
    let a = StructA { x: 10, y: 20, z: 301, e: Enum::A(0) };
    let b = StructB { a, x: 30, y: 111 };
    let x = match_nested(b);
    assert(x == 60);

    // a_x: 11, a_y: 22, b_x: 33
    let a = StructA { x: 102, y: 11, z: 22, e: Enum::B((0, 0)) };
    let b = StructB { a, x: 33, y: 111 };
    let x = match_nested(b);
    assert(x == 66);

    // a_x: 100, a_y: 200, b_x: 300
    let a = StructA { x: 103, y: 203, z: 100, e: Enum::A(200) };
    let b = StructB { a, x: 300, y: 111 };
    let x = match_nested(b);
    assert(x == 600);

    // a_x: 100, a_y: 200, b_x: 300
    let a = StructA { x: 103, y: 203, z: 100, e: Enum::B((0, 200)) };
    let b = StructB { a, x: 300, y: 111 };
    let x = match_nested(b);
    assert(x == 600);

    // a_x: 100, a_y: 200, b_x: 300
    let a = StructA { x: 103, y: 203, z: 100, e: Enum::C((0, 0, 200)) };
    let b = StructB { a, x: 300, y: 111 };
    let x = match_nested(b);
    assert(x == 600);

    // Second OR variant.
    
    // a_x: 10, a_y: 20, b_x: 30
    let a = StructA { x: 30, y: 201, z: 301, e: Enum::A(20) };
    let b = StructB { a, x: 111, y: 10 };
    let x = match_nested(b);
    assert(x == 60);
    
    // a_x: 10, a_y: 20, b_x: 30
    let a = StructA { x: 30, y: 201, z: 301, e: Enum::B((20, 0)) };
    let b = StructB { a, x: 111, y: 10 };
    let x = match_nested(b);
    assert(x == 60);
    
    // a_x: 10, a_y: 20, b_x: 30
    let a = StructA { x: 30, y: 201, z: 301, e: Enum::C((20, 0, 0)) };
    let b = StructB { a, x: 111, y: 10 };
    let x = match_nested(b);
    assert(x == 60);
    
    // a_x: 11, a_y: 22, b_x: 33
    let a = StructA { x: 102, y: 33, z: 22, e: Enum::A(0) };
    let b = StructB { a, x: 111, y: 11 };
    let x = match_nested(b);
    assert(x == 66);
    
    // a_x: 100, a_y: 200, b_x: 300
    let a = StructA { x: 200, y: 203, z: 300, e: Enum::A(0) };
    let b = StructB { a, x: 111, y: 100 };
    let x = match_nested(b);
    assert(x == 600);

    // No match. Catch-all.

    let a = StructA { x: 1234, y: 1234, z: 1234, e: Enum::A(1234) };
    let b = StructB { a, x: 1234, y: 1234 };
    let x = match_nested(b);
    assert(x == 9999);

    42
}
