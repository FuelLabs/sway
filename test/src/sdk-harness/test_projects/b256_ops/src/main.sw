script;

use std::assert::assert;
use std::b256_ops::*;

fn main() -> bool {
    let one = 1;
    let two = 2;
    let three = 3;
    let four = 4;

    let test_val: b256 = 0x0000000000000001_0000000000000002_0000000000000003_0000000000000004;

    let composed = compose(one, two, three, four);
    assert(composed == test_val);

    let(w1, w2, w3, w4) = decompose(test_val);
    assert(w1 == one);
    assert(w2 == two);
    assert(w3 == three);
    assert(w4 == four);

    let a = 0x1000000000000001_1000000000000001_1000000000000001_1000000000000001;
    let b = 0x0000000100000001_0000000010000001_0000000010000001_0000000010000001;

    let c = 0x0000000000000001_0000000000000001_0000000000000001_0000000000000001;
    let d = 0x1000000100000001_1000000010000001_1000000010000001_1000000010000001;
    let e = 0x1000000100000000_1000000010000000_1000000010000000_1000000010000000;
    let f = 0x1000000000000000_1000000000000000_1000000000000000_1000000000000000;
    let addr_1 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let addr_2 = 0x3333333333333333333333333333333333333333333333333333333333333333;
    let addr_3 = 0x1111111111111111111111111111111111111111111111111111111111111111;
    let addr_4 = 0x2222222222222222222222222222222222222222222222222222222222222222;


    assert(a & b == c);
    assert(a & c == c);
    assert(a & d == a);
    assert(a & e == f);
    assert(f & e == f);
    assert(addr_1 & addr_2 == addr_2);
    assert(addr_4 & addr_3 == 0);
    assert(addr_1 & addr_4 == addr_4);


    assert(a | b == d);
    assert(a | d == d);
    assert(a | c == a);
    assert(c | f == a);
    assert(c | e == d);
    assert(addr_1 | addr_2 == addr_1);
    assert(addr_2 | addr_3 == addr_2);
    assert(addr_3 | addr_4 == addr_2);



    assert(a ^ b == e);

    true
}

