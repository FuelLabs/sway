script;

// Tests nested struct destructuring

fn main() -> u64 {
    let tuple_in_struct = TupleInStruct {
        nested_tuple: (42u64, (42u32, (true, "ok") ) ),
    };
    let TupleInStruct { nested_tuple: (a, (b, (c, d) ) ) } = tuple_in_struct;

    let struct_in_tuple = (Point { x: 2, y: 4, }, Point { x: 3, y: 6 });
    let (Point { x: x0, y: y0 }, Point { x: x1, y: y1 }) = struct_in_tuple;

    let point1 = Point { x: 0, y: 0 };
    let point2 = Point { x: 1, y: 1 };
    let line = Line { p1: point1, p2: point2 };
    let Line { p1: Point { x: x2, y: y2 }, p2: Point { x: x3, y: y3} } = line;
    x2
}

struct Point {
    x: u64,
    y: u64,
}

struct Line {
    p1: Point,
    p2: Point,
}

struct TupleInStruct {
    nested_tuple: (u64, (u32, (bool, str[2]) ) ),
}
