script;
// This tests function, tuple, struct arguments are evaluated from left to right

fn func(a: u64, b: u64, c: u64, d: u64) -> u64 {
    d
}

struct MyStruct {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

fn main() -> bool {
    let mut x: u64 = 0;

    // function arguments evaluation
    let func_res =
        func(
            {
                x = 1;
                0
            },
            {
                x = 2;
                0
            },
            {
                x = 3;
                0
            },
            x
        );

    // tuple evaluation
    x = 0;
    let tuple_res =
        (
            {
                x = 1;
                0
            },
            {
                x = 2;
                1
            },
            {
                x = 3;
                2
            },
            x
        );

    // struct evaluation
    x = 0;
    let struct_res =
        MyStruct {
            a: {
                x = 1;
                0
            },
            b: {
                x = 2;
                1
            },
            c: {
                x = 3;
                2
            },
            d: x
        };

    return (func_res == 3) && (tuple_res.3 == 3) && (struct_res.d == 3);
}