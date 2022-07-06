library struct_destructuring;

fn struct_destructuring() {
    let point1 = Point {
        x: 0,
        y: 0
    };
    // Destructure the values from the struct into variables
    let Point {
        x, y
    } = point1;

    let point2 = Point {
        x: 1,
        y: 1
    };
    // If you do not care about specific struct fields then use ".." at the end of your variable list
    let Point {
        x, ..
    } = point2;

    let line = Line {
        p1: point1,
        p2: point2
    };
    // Destructure the vaues from the nested structs into variables
    let Line {
        p1: Point {
            x: x0, y: y0
        },
        p2: Point {
            x: x1, y: y1
        }
    } = line;
}

struct Point {
    x: u64,
    y: u64,
}

struct Line {
    p1: Point,
    p2: Point,
}