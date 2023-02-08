contract;

struct Point {
    x: u32,
    y: u32,
}

/// A simple function declaration with a struct as a return type
fn foo() -> Point {
    Point { x: 1, y: 2 }
}

/// A function declaration with struct as a function parameter
pub fn bar(p: Point) -> Point {
    Point { x: p.x, y: p.y }
}

// Function expressions
fn test() {
    let p = foo();
    let p2 = bar(p);
}

pub enum Rezult<T, E> {
    Ok: T,
    Err: E,
}

pub enum DumbError {
    Error: (),
}

// Function with generic types
pub fn func(r: Rezult<u8, DumbError>) -> Rezult<u8, DumbError> {
    Rezult::Ok(1u8)
}
