script;

struct S {
    foo: u64,
    bar: u64,
}

fn main() -> bool {
    let a: [bool;
    5] = [true, true, true, false, true];
    let b: [u32;
    10] = [3;
    10];
    let c = [0x01, 0x02, 0x03];
    let d = [0;
    10];
    let e: [[u64;
    4];
    2] = [[1, 2, 3, 4], [5, 6, 7, 8]];
    //let f: [u64; 1 + 1] = [0, 0];
    let g = [S {
        foo: 10, bar: 20
    },
    S {
        foo: 1, bar: 2
    }
    ];
    let h = i()[2];

    b[0] == b[9] && e[0][1] + e[1][2] == 9 && g[0].foo + g[1].bar == 12 && j(g) && /* a.len() == 5 && */
    true
}

fn i() -> [u64;
4] {
    [0, 1, 2, 3]
}

fn j(ary_arg: [S;
2]) -> bool {
    ary_arg[0].foo + ary_arg[1].bar == 12
}
