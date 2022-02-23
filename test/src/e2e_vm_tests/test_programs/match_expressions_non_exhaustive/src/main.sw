script;

struct Point {
    x: u64,
    y: u64
}

struct CrazyPoint {
    p1: Point,
    p2: Point
}

fn main() -> u64 {
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

    let p = Point {
        x: 3,
        y: 4,
    };
    // should fail
    let foo = match p {
        Point { x: 3, y } => { y },
        Point { x: 3, y: 4 } => { 24 },
    };
    // should succeed
    let foo = match p {
        Point { x: 3, y } => { y },
        Point { x: 3, y: 4 } => { 24 },
        Point { x, y } => { x },
    };
    // should succeed
    let foo = match p {
        Point { x: 3, y } => { y },
        Point { x: 3, y: 4 } => { 24 },
        a => { 24 },
    };
    // should succeed
    let foo = match p {
        Point { x: 3, y } => { y },
        Point { x: 3, y: 4 } => { 24 },
        _ => { 24 },
    };

    let p = CrazyPoint {
        p1: Point {
            x: 100,
            y: 200
        },
        p2: Point {
            x: 300,
            y: 400
        }
    };
    // should fail
    let foo = match p {
        CrazyPoint { p1: Point { x: 0, y: 1 }, p2 } => { 42 },
    };

    42u64
}
