library;

mod data_structures;
use data_structures::{Foo, Line, Point, TupleInStruct};

fn hardcoded_instantiation() -> Foo {
    // Instantiate `foo` as `Foo`
    let mut foo = Foo {
        bar: 42,
        baz: false,
    };

    // Access and write to "baz"
    foo.baz = true;

    // Return the struct
    foo
}

fn variable_instantiation() -> Foo {
    // Declare variables with the same names as the fields in `Foo`
    let number = 42;
    let truthness = false;

    // Instantiate `foo` as `Foo`
    let mut foo = Foo {
        bar: number,
        baz: truthness,
    };

    // Access and write to "baz"
    foo.baz = true;

    // Return the struct
    foo
}

fn shorthand_instantiation() -> Foo {
    // Declare variables with the same names as the fields in `Foo`
    let bar = 42;
    let baz = false;

    // Instantiate `foo` as `Foo`
    let mut foo = Foo { bar, baz };

    // Access and write to "baz"
    foo.baz = true;

    // Return the struct
    foo
}

fn struct_destructuring() {
    let point1 = Point { x: 0, y: 0 };
    // Destructure the values from the struct into variables
    let Point { x, y } = point1;

    let point2 = Point { x: 1, y: 1 };
    // If you do not care about specific struct fields then use ".." at the end of your variable list
    let Point { x, .. } = point2;

    let line = Line {
        p1: point1,
        p2: point2,
    };
    // Destructure the values from the nested structs into variables
    let Line {
        p1: Point { x: x0, y: y0 },
        p2: Point { x: x1, y: y1 },
    } = line;
    // You may also destructure tuples nested in structs and structs nested in tuples
    let tuple_in_struct = TupleInStruct {
        nested_tuple: (42u64, (42u32, (true, "ok"))),
    };
    let TupleInStruct {
        nested_tuple: (a, (b, (c, d))),
    } = tuple_in_struct;

    let struct_in_tuple = (Point { x: 2, y: 4 }, Point { x: 3, y: 6 });
    let (Point { x: x0, y: y0 }, Point { x: x1, y: y1 }) = struct_in_tuple;

    let e = line;


}

fn main() {
    // ANCHOR: vec_new
    let v: Vec<u64> = Vec::new();
    // ANCHOR_END: vec_new
    // ANCHOR: vec_push
    let mut v = Vec::new();

    v.push(5);
    v.push(6);
    v.push(7);
    v.push(8);
    // ANCHOR_END: vec_push
    // ANCHOR: vec_get
    let third = v.get(2);
    match third {
        Some(third) => log(third),
        None => revert(42),
    }
    // ANCHOR_END: vec_get
    // ANCHOR: vec_get_oob
    let does_not_exist = v.get(100);
    // ...decide here how to handle an out-of-bounds access
    // ANCHOR_END: vec_get_oob
    // ANCHOR: vec_iterate
    let mut i = 0;
    while i < v.len() {
        log(v.get(i).unwrap());
        i += 1;
    }
    // ANCHOR_END: vec_iterate
    // ANCHOR: vec_multiple_data_types
    enum TableCell {
        Int: u64,
        B256: b256,
        Boolean: bool,
    }

    let mut row = Vec::new();
    row.push(TableCell::Int(3));
    row.push(TableCell::B256(0x0101010101010101010101010101010101010101010101010101010101010101));
    row.push(TableCell::Boolean(true));
    // ANCHOR_END: vec_multiple_data_types
}
