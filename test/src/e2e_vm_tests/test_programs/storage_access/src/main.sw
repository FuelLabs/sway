contract;

use std::constants::ETH_ID;

struct S {
    a: u64,
    b: u64,
    c: b256,
    t: T
}

struct T {
    a: u64,
    b: u64,
    c: b256
}

storage {
    x: u64 = 0,
    y: b256 = ETH_ID,
    s: S = S {
        a: 0,
        b: 0,
        c: ETH_ID,
        t: T {
            a: 0,
            b: 0,
            c: ETH_ID,
        }
    }
}

abi TestAbi {
    fn get_x() -> u64;
    fn get_y() -> b256;
    fn get_s() -> S;
    fn get_s_dot_t() -> T;
    fn get_s_dot_t_dot_a() -> u64;
    fn get_s_dot_t_dot_b() -> u64;
    fn get_s_dot_t_dot_c() -> b256;
    fn set_x(x: u64) ;
    fn set_y(y: b256);
    fn set_s(s: S);
    fn set_s_dot_t(t: T);
    fn set_s_dot_t_dot_a(a: u64);
    fn set_s_dot_t_dot_b(b: u64);
    fn set_s_dot_t_dot_c(c: b256);
}

impl TestAbi for Contract {
    impure fn get_x() -> u64 {
        storage.x
    }
    impure fn get_y() -> b256 {
        storage.y
    }
    impure fn get_s() -> S {
        storage.s
    }
    impure fn get_s_dot_t() -> T {
        storage.s.t
    }
    impure fn get_s_dot_t_dot_a() -> u64 {
        storage.s.t.a
    }
    impure fn get_s_dot_t_dot_b() -> u64 {
        storage.s.t.b
    }
    impure fn get_s_dot_t_dot_c() -> b256 {
        storage.s.t.c
    }
    impure fn set_x(x: u64) {
        storage.x = x;
    }
    impure fn set_y(y: b256) {
        storage.y = y;
    }
    impure fn set_s(s: S) {
        storage.s = s;
    }
    impure fn set_s_dot_t(t: T) {
        storage.s.t = t;
    }
    impure fn set_s_dot_t_dot_a(a: u64) {
        storage.s.a = a;
    }
    impure fn set_s_dot_t_dot_b(b: u64) {
        storage.s.b = b;
    }
    impure fn set_s_dot_t_dot_c(c: b256) {
        storage.s.c = c;
    }
}
