script;

struct Point {
  x: u64,
  y: u64
}

struct Point3D {
  x: u64,
  y: u64,
  z: u64
}

struct Line {
    p1: Point,
    p2: Point
}

enum Kind {
    Point: Point,
    Point3D: Point3D,
    Line: Line
}

fn main() -> u64 {
    let p = Point {
        x: 1u64,
        y: 2u64,
    };

    match p {
        Point { x, .. } => { x },
    };

    let p2 = Point3D {
        x: 1u64,
        y: 2u64,
        z: 3u64,
    };

    match p2 {
        Point3D { x, .. } => { x },
    };

    let l = Line { p1: p, p2: p };

    match l {
        Line { p1, .. } => {}
    }

    match l {
        Line { p1: Point { .. }, p2: Point { .. } } => {}
    }

    let k = Kind::Point(p);

    match k {
        Kind::Point(Point { x, .. }) => {},
        Kind::Point3D(Point3D { z, .. }) => {},
        Kind::Line(Line { p1: Point { .. }, p2: Point { .. } }) => {},
    }

    0
}
