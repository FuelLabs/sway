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

fn match_point(p: Point) -> u64 {
    match p {
        Point { x: 11, .. } => { 11 },
        Point { y: 22, .. } => { 22 },
        Point { x: 111, .. } | Point { y: 222, .. } => { 111222 },
        Point { x: 333 | 444, .. } | Point { y: 555 | 666, .. } => { 3456 },
        Point { x, .. } => { x },
    }
}

fn match_point_3d(p: Point3D) -> u64 {
    match p {
        Point3D { x: 11, y, .. } => { y },
        Point3D { y: 22, x: y, .. } => { y },
        Point3D { x: 111, z, .. } | Point3D { y: 222, x: z, .. } => { z },
        Point3D { z, .. } => { z },
        _ => 9999, // TODO: Remove once bugs in exhaustiveness algorithm are fixed: Non-exhaustive match expression. Missing patterns `Point3D { x: _, y: _ }`
    }
}

fn main() -> u64 {
    let m = match_point(Point { x: 11, y: 0 });
    assert(m == 11);

    let m = match_point(Point { x: 0, y: 22 });
    assert(m == 22);

    let m = match_point(Point { x: 111, y: 0 });
    assert(m == 111222);

    let m = match_point(Point { x: 0, y: 222 });
    assert(m == 111222);

    let m = match_point(Point { x: 333, y: 0 });
    assert(m == 3456);

    let m = match_point(Point { x: 444, y: 0 });
    assert(m == 3456);

    let m = match_point(Point { x: 0, y: 555 });
    assert(m == 3456);

    let m = match_point(Point { x: 0, y: 666 });
    assert(m == 3456);

    let m = match_point(Point { x: 42, y: 0 });
    assert(m == 42);

    let m = match_point_3d(Point3D { x: 11, y: 42, z: 0 });
    assert(m == 42);

    let m = match_point_3d(Point3D { x: 42, y: 22, z: 0 });
    assert(m == 42);

    let m = match_point_3d(Point3D { x: 111, y: 0, z: 42 });
    assert(m == 42);

    let m = match_point_3d(Point3D { x: 42, y: 222, z: 0 });
    assert(m == 42);
    
    let m = match_point_3d(Point3D { x: 0, y: 0 , z: 42});
    assert(m == 42);

    let l = Line { p1: Point { x: 0, y: 0 }, p2: Point { x: 1, y: 1 } };
    match l {
        Line { p1: _p1, .. } => {}
    }

    match l {
        Line { p1: Point { .. }, p2: Point { .. } } => {}
    }

    let k = Kind::Point(Point { x: 0, y: 0 });

    match k {
        Kind::Point(Point { x: _x, .. }) => {},
        Kind::Point3D(Point3D { z: _z, .. }) => {},
        Kind::Line(Line { p1: Point { .. }, p2: Point { .. } }) => {},
    }

    42
}
