script;

use std::assert::assert;

struct S {
    foo: u64,
    bar: u64,
}

fn main() -> bool {
    let _a: [bool; 5] = [true, true, true, false, true];
    let b: [u32; 10] = [3; 10];
    let _c = [0x01, 0x02, 0x03];
    let _d = [0; 10];
    let e: [[u64; 4]; 2] = [[1, 2, 3, 4], [5, 6, 7, 8]];
    let g = [S {
    foo: 10,
    bar: 20,
}, S { foo: 1, bar: 2 }];
    let _h = i()[2];

    assert(test_init() == 110);

    b[0] == b[9] && e[0][1] + e[1][2] == 9 && g[0].foo + g[1].bar == 12 && j(g) && /* a.len() == 5 && */ true
}

fn i() -> [u64; 4] {
    [0, 1, 2, 3]
}

fn j(ary_arg: [S; 2]) -> bool {
    ary_arg[0].foo + ary_arg[1].bar == 12
}

fn test_init() -> u64 {
    let mut a: [u64; 10] = [11; 10];

    let mut i = 0;
    let mut m = 0;
    while i < 10 {
        m += a[i];
        i += 1;
    }
    m
}
