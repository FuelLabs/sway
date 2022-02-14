script;

struct Point {
    x: u64,
    y: u64
}

fn main() -> u64 {
    /*
    let p = Point {
        x: 3,
        y: 4,
    };

    match p {
        Point { x: 3, y } => { y },
        Point { x: 3, y: 4 } => { 24 },
        _ => { 24 },
    }
    */

    let x = 0;
    // should fail
    let y = match x {
        0 => { 0 },
        10 => { 0 },
        5 => { 0 },
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

    /*
    let x = (1, 2);
    // should fail
    let y = match x {
        (0, 0) => { 0 },
        (1, 1) => { 0 },
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
    */

    42u64
}
