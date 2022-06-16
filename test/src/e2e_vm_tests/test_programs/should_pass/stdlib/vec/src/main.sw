script;

use std::{assert::assert, hash::sha256, option::Option, revert::revert, vec::Vec};

struct S {
    x: u32,
    y: b256,
}

enum E {
    X: (),
    Y: b256,
    Z: (b256,
    b256), 
}

fn main() -> bool {
    test_vector_new_u8();
    test_vector_new_b256();
    test_vector_new_struct();
    test_vector_new_enum();
    test_vector_new_tuple();
    test_vector_new_string();
    test_vector_new_array();
    test_vector_with_capacity_u64();
    true
}

fn test_vector_new_u8() {
    let mut v: Vec<u8> = ~Vec::new::<u8>();

    let n0 = 0u8;
    let n1 = 1u8;
    let n2 = 2u8;
    let n3 = 3u8;
    let n4 = 4u8;
    let n5 = 5u8;
    let n6 = 6u8;
    let n7 = 7u8;
    let n8 = 8u8;

    assert(v.len() == 0);
    assert(v.capacity() == 0);
    assert(v.is_empty());

    v.push(n0);
    v.push(n1);
    v.push(n2);
    v.push(n3);
    v.push(n4);

    assert(v.len() == 5);
    assert(v.capacity() == 8);
    assert(v.is_empty() == false);

    match v.get(0) {
        Option::Some(val) => assert(val == n0), Option::None => revert(0), 
    }

    // Push after get
    v.push(n5);
    v.push(n6);
    v.push(n7);
    v.push(n8);

    match v.get(4) {
        Option::Some(val) => assert(val == n4), Option::None => revert(0), 
    }

    match v.get(n6) {
        Option::Some(val) => assert(val == n6), Option::None => revert(0), 
    }

    assert(v.len() == 9);
    assert(v.capacity() == 16);
    assert(!v.is_empty());

    // Test after capacity change
    match v.get(4) {
        Option::Some(val) => assert(val == n4), Option::None => revert(0), 
    }

    match v.get(6) {
        Option::Some(val) => assert(val == n6), Option::None => revert(0), 
    }

    v.clear();

    // Empty after clear
    assert(v.len() == 0);
    assert(v.capacity() == 16);
    assert(v.is_empty() == true);

    match v.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    v.push(n0);
    v.push(n1);
    v.push(n2);
    v.push(n3);
    v.push(n4);

    assert(v.len() == 5);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    match v.get(4) {
        Option::Some(val) => assert(val == n4), Option::None => revert(0), 
    }

    // Out of bounds access
    match v.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}

fn test_vector_new_b256() {
    let mut v: Vec<b256> = ~Vec::new::<b256>();

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;
    let b3 = 0x0000000000000000000000000000000000000000000000000000000000000003;
    let b4 = 0x0000000000000000000000000000000000000000000000000000000000000004;
    let b5 = 0x0000000000000000000000000000000000000000000000000000000000000005;
    let b6 = 0x0000000000000000000000000000000000000000000000000000000000000006;
    let b7 = 0x0000000000000000000000000000000000000000000000000000000000000007;
    let b8 = 0x0000000000000000000000000000000000000000000000000000000000000008;

    assert(v.len() == 0);
    assert(v.capacity() == 0);
    assert(v.is_empty() == true);

    v.push(b0);
    v.push(b1);
    v.push(b2);
    v.push(b3);
    v.push(b4);

    assert(v.len() == 5);
    assert(v.capacity() == 8);
    assert(v.is_empty() == false);

    match v.get(0) {
        Option::Some(val) => assert(val == b0), Option::None => revert(0), 
    }

    // Push after get
    v.push(b5);
    v.push(b6);
    v.push(b7);
    v.push(b8);

    match v.get(4) {
        Option::Some(val) => assert(val == b4), Option::None => revert(0), 
    }

    match v.get(6) {
        Option::Some(val) => assert(val == b6), Option::None => revert(0), 
    }

    assert(v.len() == 9);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    // Test after capacity change
    match v.get(4) {
        Option::Some(val) => assert(val == b4), Option::None => revert(0), 
    }

    match v.get(6) {
        Option::Some(val) => assert(val == b6), Option::None => revert(0), 
    }

    v.clear();

    // Empty after clear
    assert(v.len() == 0);
    assert(v.capacity() == 16);
    assert(v.is_empty() == true);

    match v.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    v.push(b0);
    v.push(b1);
    v.push(b2);
    v.push(b3);
    v.push(b4);

    assert(v.len() == 5);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    match v.get(4) {
        Option::Some(val) => assert(val == b4), Option::None => revert(0), 
    }

    // Out of bounds access
    match v.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}

fn test_vector_new_struct() {
    let mut v: Vec<S> = ~Vec::new::<S>();

    let n0 = 0u32;
    let n1 = 1u32;
    let n2 = 2u32;
    let n3 = 3u32;
    let n4 = 4u32;
    let n5 = 5u32;
    let n6 = 6u32;
    let n7 = 7u32;
    let n8 = 8u32;

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;
    let b3 = 0x0000000000000000000000000000000000000000000000000000000000000003;
    let b4 = 0x0000000000000000000000000000000000000000000000000000000000000004;
    let b5 = 0x0000000000000000000000000000000000000000000000000000000000000005;
    let b6 = 0x0000000000000000000000000000000000000000000000000000000000000006;
    let b7 = 0x0000000000000000000000000000000000000000000000000000000000000007;
    let b8 = 0x0000000000000000000000000000000000000000000000000000000000000008;

    assert(v.len() == 0);
    assert(v.capacity() == 0);
    assert(v.is_empty() == true);

    v.push(S {
        x: n0, y: b0
    });
    v.push(S {
        x: n1, y: b1
    });
    v.push(S {
        x: n2, y: b2
    });
    v.push(S {
        x: n3, y: b3
    });
    v.push(S {
        x: n4, y: b4
    });

    assert(v.len() == 5);
    assert(v.capacity() == 8);
    assert(v.is_empty() == false);

    match v.get(0) {
        Option::Some(val) => {
            assert(val.x == n0);
            assert(val.y == b0);
        },
        Option::None => revert(0), 
    }

    // Push after get
    v.push(S {
        x: n5, y: b5
    });
    v.push(S {
        x: n6, y: b6
    });
    v.push(S {
        x: n7, y: b7
    });
    v.push(S {
        x: n8, y: b8
    });

    match v.get(4) {
        Option::Some(val) => {
            assert(val.x == n4);
            assert(val.y == b4);
        },
        Option::None => revert(0), 
    }

    match v.get(6) {
        Option::Some(val) => {
            assert(val.x == n6);
            assert(val.y == b6);
        },
        Option::None => revert(0), 
    }

    assert(v.len() == 9);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    // Test after capacity change
    match v.get(4) {
        Option::Some(val) => {
            assert(val.x == n4);
            assert(val.y == b4);
        },
        Option::None => revert(0), 
    }

    match v.get(6) {
        Option::Some(val) => {
            assert(val.x == n6);
            assert(val.y == b6);
        },
        Option::None => revert(0), 
    }

    v.clear();

    // Empty after clear
    assert(v.len() == 0);
    assert(v.capacity() == 16);
    assert(v.is_empty() == true);

    match v.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    v.push(S {
        x: n0, y: b0
    });
    v.push(S {
        x: n1, y: b1
    });
    v.push(S {
        x: n2, y: b2
    });
    v.push(S {
        x: n3, y: b3
    });
    v.push(S {
        x: n4, y: b4
    });

    assert(v.len() == 5);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    match v.get(4) {
        Option::Some(val) => {
            assert(val.x == n4);
            assert(val.y == b4);
        },
        Option::None => revert(0), 
    }

    // Out of bounds access
    match v.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}

fn test_vector_new_enum() {
    let mut v: Vec<E> = ~Vec::new::<E>();

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;

    assert(v.len() == 0);
    assert(v.capacity() == 0);
    assert(v.is_empty() == true);

    v.push(E::Y(b0));
    v.push(E::X);
    v.push(E::Z((b1, b2)));

    assert(v.len() == 3);
    assert(v.capacity() == 4);
    assert(v.is_empty() == false);

    match v.get(0) {
        Option::Some(val) => {
            match val {
                E::Y(b) => assert(b == b0), _ => revert(0), 
            }
        },
        Option::None => revert(0), 
    }

    match v.get(1) {
        Option::Some(val) => {
            match val {
                E::X => {
                },
                _ => revert(0), 
            }
        },
        Option::None => revert(0), 
    }

    match v.get(2) {
        Option::Some(val) => {
            match val {
                E::Z(t) => {
                    assert(t.0 == b1);
                    assert(t.1 == b2);
                },
                _ => revert(0), 
            }
        },
        Option::None => revert(0), 
    }
}

fn test_vector_new_tuple() {
    let mut v: Vec<(u16, b256)> = ~Vec::new::<(u16, b256)>();

    let n0 = 0u16;
    let n1 = 1u16;
    let n2 = 2u16;
    let n3 = 3u16;
    let n4 = 4u16;
    let n5 = 5u16;
    let n6 = 6u16;
    let n7 = 7u16;
    let n8 = 8u16;

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;
    let b3 = 0x0000000000000000000000000000000000000000000000000000000000000003;
    let b4 = 0x0000000000000000000000000000000000000000000000000000000000000004;
    let b5 = 0x0000000000000000000000000000000000000000000000000000000000000005;
    let b6 = 0x0000000000000000000000000000000000000000000000000000000000000006;
    let b7 = 0x0000000000000000000000000000000000000000000000000000000000000007;
    let b8 = 0x0000000000000000000000000000000000000000000000000000000000000008;

    assert(v.len() == 0);
    assert(v.capacity() == 0);
    assert(v.is_empty() == true);

    v.push((n0, b0));
    v.push((n1, b1));
    v.push((n2, b2));
    v.push((n3, b3));
    v.push((n4, b4));

    assert(v.len() == 5);
    assert(v.capacity() == 8);
    assert(v.is_empty() == false);

    match v.get(0) {
        Option::Some(val) => {
            assert(val.0 == n0);
            assert(val.1 == b0);
        },
        Option::None => revert(0), 
    }

    // Push after get
    v.push((n5, b5));
    v.push((n6, b6));
    v.push((n7, b7));
    v.push((n8, b8));

    match v.get(4) {
        Option::Some(val) => {
            assert(val.0 == n4);
            assert(val.1 == b4);
        },
        Option::None => revert(0), 
    }

    match v.get(6) {
        Option::Some(val) => {
            assert(val.0 == n6);
            assert(val.1 == b6);
        },
        Option::None => revert(0), 
    }

    assert(v.len() == 9);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    // Test after capacity change
    match v.get(4) {
        Option::Some(val) => {
            assert(val.0 == n4);
            assert(val.1 == b4);
        },
        Option::None => revert(0), 
    }

    match v.get(6) {
        Option::Some(val) => {
            assert(val.0 == n6);
            assert(val.1 == b6);
        },
        Option::None => revert(0), 
    }

    v.clear();

    // Empty after clear
    assert(v.len() == 0);
    assert(v.capacity() == 16);
    assert(v.is_empty() == true);

    match v.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    v.push((n0, b0));
    v.push((n1, b1));
    v.push((n2, b2));
    v.push((n3, b3));
    v.push((n4, b4));

    assert(v.len() == 5);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    match v.get(4) {
        Option::Some(val) => {
            assert(val.0 == n4);
            assert(val.1 == b4);
        },
        Option::None => revert(0), 
    }

    // Out of bounds access
    match v.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}

fn test_vector_new_string() {
    let mut v: Vec<str[4]> = ~Vec::new::<str[4]>();

    let s0 = "fuel";
    let s1 = "john";
    let s2 = "nick";

    assert(v.len() == 0);
    assert(v.capacity() == 0);
    assert(v.is_empty() == true);

    v.push(s0);
    v.push(s1);
    v.push(s2);

    assert(v.len() == 3);
    assert(v.capacity() == 4);
    assert(v.is_empty() == false);

    // Can't compare strings directly. Compare their hashes instead.
    match v.get(0) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s0));
        },
        Option::None => revert(0), 
    }

    match v.get(1) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s1));
        },
        Option::None => revert(0), 
    }

    match v.get(2) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s2));
        },
        Option::None => revert(0), 
    }
}

fn test_vector_new_array() {
    let mut v: Vec<[u64;
    3]> = ~Vec::new::<[u64;
    3]>();

    let a0 = [0, 1, 2];
    let a1 = [3, 4, 5];
    let a2 = [6, 7, 8];

    assert(v.len() == 0);
    assert(v.capacity() == 0);
    assert(v.is_empty() == true);

    v.push(a0);
    v.push(a1);
    v.push(a2);

    assert(v.len() == 3);
    assert(v.capacity() == 4);
    assert(v.is_empty() == false);

    // Can't compare strings directly. Compare their hashes instead.
    match v.get(0) {
        Option::Some(val) => {
            assert(val[0] == a0[0]);
            assert(val[1] == a0[1]);
            assert(val[2] == a0[2]);
        },
        Option::None => revert(0), 
    }

    match v.get(1) {
        Option::Some(val) => {
            assert(val[0] == a1[0]);
            assert(val[1] == a1[1]);
            assert(val[2] == a1[2]);
        },
        Option::None => revert(0), 
    }

    match v.get(2) {
        Option::Some(val) => {
            assert(val[0] == a2[0]);
            assert(val[1] == a2[1]);
            assert(val[2] == a2[2]);
        },
        Option::None => revert(0), 
    }
}

fn test_vector_with_capacity_u64() {
    let mut v: Vec<u64> = ~Vec::with_capacity::<u64>(8);

    let n0 = 0;
    let n1 = 1;
    let n2 = 2;
    let n3 = 3;
    let n4 = 4;
    let n5 = 5;
    let n6 = 6;
    let n7 = 7;
    let n8 = 8;

    assert(v.len() == 0);
    assert(v.capacity() == 8);
    assert(v.is_empty() == true);

    v.push(n0);
    v.push(n1);
    v.push(n2);
    v.push(n3);
    v.push(n4);

    assert(v.len() == 5);
    assert(v.capacity() == 8);
    assert(v.is_empty() == false);

    match v.get(0) {
        Option::Some(val) => assert(val == n0), Option::None => revert(0), 
    }

    // Push after get
    v.push(n5);
    v.push(n6);
    v.push(n7);
    v.push(n8);

    match v.get(4) {
        Option::Some(val) => assert(val == n4), Option::None => revert(0), 
    }

    match v.get(6) {
        Option::Some(val) => assert(val == n6), Option::None => revert(0), 
    }

    assert(v.len() == 9);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    v.clear();

    // Empty after clear
    assert(v.len() == 0);
    assert(v.capacity() == 16);
    assert(v.is_empty() == true);

    match v.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    v.push(n0);
    v.push(n1);
    v.push(n2);
    v.push(n3);
    v.push(n4);

    assert(v.len() == 5);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    match v.get(4) {
        Option::Some(val) => assert(val == n4), Option::None => revert(0), 
    }

    // Out of bounds access
    match v.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}
