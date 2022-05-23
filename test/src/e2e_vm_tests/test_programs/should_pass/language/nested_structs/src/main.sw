script;
use std::assert::assert;

struct I {
    zero: u64,
    one: u8
}

struct T6 {
    zero: u8,
    one: u8,
    two: I,
    three: u16,
    four: (u8, u8)
}

struct W {
    t5: u64,
    t6: T6
}

struct T3 {
    zero: u8,
    one: u8
}

struct T {
    t3: T3,
    t4: u16
}

struct T1 {
    zero: u64,
    one: u64
}

struct S {
    t0: W,
    t1: T1,
    t2: T,
    t3: u16
}

fn main() -> bool {
    let mut s = S {
        t0: W {
            t5: 5,
            t6: T6 {
                zero: 6,
                one: 7, 
                two: I {
                    zero: 8, 
                    one: 9
                },
                three: 10,
                four: (11, 12)
            }
        },
        t1: T1 {
            zero: 0, 
            one: 1
        },
        t2: T {
            t3: T3 {
                zero: 2, 
                one: 3
            },
            t4: 4
        },
        t3: 13
    };

    assert(s.t1.zero == 0);
    assert(s.t1.one == 1);
    assert(s.t2.t3.zero == 2);
    assert(s.t2.t3.one == 3);
    assert(s.t2.t4 == 4);
    assert(s.t0.t5 == 5);
    assert(s.t0.t6.zero == 6);
    assert(s.t0.t6.one == 7);
    assert(s.t0.t6.two.zero == 8);
    assert(s.t0.t6.two.one == 9);
    assert(s.t0.t6.three == 10);
    assert((s.t0.t6.four).0 == 11);
    assert((s.t0.t6.four).1 == 12);
    assert(s.t3 == 13);

    s.t1.zero = 10;
    s.t1.one = 11;
    s.t2.t3.zero = 12;
    s.t2.t3.one = 13;
    s.t2.t4 = 14;
    s.t0.t5 = 15;
    s.t0.t6.zero = 16;
    s.t0.t6.one = 17;
    s.t0.t6.two.zero = 18;
    s.t0.t6.two.one = 19;
    s.t0.t6.three = 110;
    s.t0.t6.four = (111, 112);
    s.t3 = 113;

    assert(s.t1.zero == 10);
    assert(s.t1.one == 11);
    assert(s.t2.t3.zero == 12);
    assert(s.t2.t3.one == 13);
    assert(s.t2.t4 == 14);
    assert(s.t0.t5 == 15);
    assert(s.t0.t6.zero == 16);
    assert(s.t0.t6.one == 17);
    assert(s.t0.t6.two.zero == 18);
    assert(s.t0.t6.two.one == 19);
    assert(s.t0.t6.three == 110);
    assert((s.t0.t6.four).0 == 111);
    assert((s.t0.t6.four).1 == 112);
    assert(s.t3 == 113);

    true
}
