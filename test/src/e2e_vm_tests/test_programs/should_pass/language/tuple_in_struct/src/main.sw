script;

struct W {
    t5: u64,
    t6: (u8, u8, (u64, u8), u16),
}

struct T {
    t3: (u8, u8),
    t4: u16
}

struct S {
    t0: W,
    t1: (u64, u64),
    t2: T,
}

struct U {
    u: u64
}

fn main() -> bool {
    let s = S {
        t0: W {
            t5: 5,
            t6: (6, 7, (8, 9), 10)
        },
        t1: (0, 1),
        t2: T {
            t3: (2, 3),
            t4: 4
        }
    };
    
    assert((s.t1).0 == 0);
    assert((s.t1).1 == 1);
    assert((s.t2.t3).0 == 2);
    assert((s.t2.t3).1 == 3);
    assert(s.t2.t4 == 4);
    assert(((s.t0).t5) == 5);
    assert(((s.t0).t6).0 == 6);
    assert(((s.t0).t6).1 == 7);
    assert((((s.t0).t6).2).0 == 8);
    assert((((s.t0).t6).2).1 == 9);
    assert(((s.t0).t6).3 == 10);

    let u = U {
        u: 22 
    };
    assert(u.u == 22);

    true
}
