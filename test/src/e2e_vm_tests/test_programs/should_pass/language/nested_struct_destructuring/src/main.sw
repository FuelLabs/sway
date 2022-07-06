script;

// Tests nested struct destructuring

fn main() -> u64 {
    let point1 = Point { x: 0, y: 0 };
    let point2 = Point { x: 1, y: 1 };
    let line = Line { p1: point1, p2: point2 };
    let Line { p1: Point { x: x0, y: y0 }, p2: Point { x: x1, y: y1} } = line;
    x0
}

struct Point {
    x: u64,
    y: u64,
}

struct Line {
    p1: Point,
    p2: Point,
}
