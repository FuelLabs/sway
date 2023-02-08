contract;

struct Point {
    x: u32,
    y: u32,
}

/// A simple function declaration with a struct as a return type
fn foo() -> Point {
    Point { x: 1, y: 2 }
}

/// A function declaration with struct as parameters
pub fn bar(p: Point) -> Point {
    Point { x: p.x, y: p.y }
}

// Function expressions
fn test() {
    let p = foo();
    let p2 = bar(p);
}
