library adt_tests;

struct Point {
    x: u64,
    y: u64
}

pub fn point_test() {
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
}

struct CrazyPoint {
    p1: Point,
    p2: Point
}

pub fn crazy_point_test() {
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
}
