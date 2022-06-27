script;

use std::{assert::assert, hash::sha256, option::Option, revert::revert, vec::Vec};

struct SimpleStruct {
    x: u32,
    y: b256,
}

enum SimpleEnum {
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
    let mut vector: Vec<u8> = ~Vec::new();

    let number0 = 0u8;
    let number1 = 1u8;
    let number2 = 2u8;
    let number3 = 3u8;
    let number4 = 4u8;
    let number5 = 5u8;
    let number6 = 6u8;
    let number7 = 7u8;
    let number8 = 8u8;

    assert(vector.len() == 0);
    assert(vector.capacity() == 0);
    assert(vector.is_empty());

    vector.push(number0);
    vector.push(number1);
    vector.push(number2);
    vector.push(number3);
    vector.push(number4);
    vector.push(false);

    assert(vector.len() == 5);
    assert(vector.capacity() == 8);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => assert(val == number0), Option::None => revert(0), 
    }

    // Push after get
    vector.push(number5);
    vector.push(number6);
    vector.push(number7);
    vector.push(number8);

    match vector.get(4) {
        Option::Some(val) => assert(val == number4), Option::None => revert(0), 
    }

    match vector.get(number6) {
        Option::Some(val) => assert(val == number6), Option::None => revert(0), 
    }

    assert(vector.len() == 9);
    assert(vector.capacity() == 16);
    assert(!vector.is_empty());

    // Test after capacity change
    match vector.get(4) {
        Option::Some(val) => assert(val == number4), Option::None => revert(0), 
    }

    match vector.get(6) {
        Option::Some(val) => assert(val == number6), Option::None => revert(0), 
    }

    vector.clear();

    // Empty after clear
    assert(vector.len() == 0);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == true);

    match vector.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    vector.push(number0);
    vector.push(number1);
    vector.push(number2);
    vector.push(number3);
    vector.push(number4);
    vector.push("this should break it 1");

    assert(vector.len() == 5);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    match vector.get(4) {
        Option::Some(val) => assert(val == number4), Option::None => revert(0), 
    }

    // Out of bounds access
    match vector.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}

fn test_vector_new_b256() {
    let mut vector: Vec<b256> = ~Vec::new::<b256>();

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;
    let b3 = 0x0000000000000000000000000000000000000000000000000000000000000003;
    let b4 = 0x0000000000000000000000000000000000000000000000000000000000000004;
    let b5 = 0x0000000000000000000000000000000000000000000000000000000000000005;
    let b6 = 0x0000000000000000000000000000000000000000000000000000000000000006;
    let b7 = 0x0000000000000000000000000000000000000000000000000000000000000007;
    let b8 = 0x0000000000000000000000000000000000000000000000000000000000000008;

    assert(vector.len() == 0);
    assert(vector.capacity() == 0);
    assert(vector.is_empty() == true);

    vector.push(b0);
    vector.push(b1);
    vector.push(b2);
    vector.push(b3);
    vector.push(b4);
    vector.push("this should break it 2");

    assert(vector.len() == 5);
    assert(vector.capacity() == 8);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => assert(val == b0), Option::None => revert(0), 
    }

    // Push after get
    vector.push(b5);
    vector.push(b6);
    vector.push(b7);
    vector.push(b8);
    vector.push("this should break it 3");

    match vector.get(4) {
        Option::Some(val) => assert(val == b4), Option::None => revert(0), 
    }

    match vector.get(6) {
        Option::Some(val) => assert(val == b6), Option::None => revert(0), 
    }

    assert(vector.len() == 9);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    // Test after capacity change
    match vector.get(4) {
        Option::Some(val) => assert(val == b4), Option::None => revert(0), 
    }

    match vector.get(6) {
        Option::Some(val) => assert(val == b6), Option::None => revert(0), 
    }

    vector.clear();

    // Empty after clear
    assert(vector.len() == 0);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == true);

    match vector.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    vector.push(b0);
    vector.push(b1);
    vector.push(b2);
    vector.push(b3);
    vector.push(b4);
    vector.push("this should break it 4");

    assert(vector.len() == 5);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    match vector.get(4) {
        Option::Some(val) => assert(val == b4), Option::None => revert(0), 
    }

    // Out of bounds access
    match vector.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}

fn test_vector_new_struct() {
    let mut vector: Vec<SimpleStruct> = ~Vec::new();

    let number0 = 0u32;
    let number1 = 1u32;
    let number2 = 2u32;
    let number3 = 3u32;
    let number4 = 4u32;
    let number5 = 5u32;
    let number6 = 6u32;
    let number7 = 7u32;
    let number8 = 8u32;

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;
    let b3 = 0x0000000000000000000000000000000000000000000000000000000000000003;
    let b4 = 0x0000000000000000000000000000000000000000000000000000000000000004;
    let b5 = 0x0000000000000000000000000000000000000000000000000000000000000005;
    let b6 = 0x0000000000000000000000000000000000000000000000000000000000000006;
    let b7 = 0x0000000000000000000000000000000000000000000000000000000000000007;
    let b8 = 0x0000000000000000000000000000000000000000000000000000000000000008;

    assert(vector.len() == 0);
    assert(vector.capacity() == 0);
    assert(vector.is_empty() == true);

    vector.push(SimpleStruct {
        x: number0, y: b0
    });
    vector.push(SimpleStruct {
        x: number1, y: b1
    });
    vector.push("this should break it 5");
    vector.push(SimpleStruct {
        x: number2, y: b2
    });
    vector.push(SimpleStruct {
        x: number3, y: b3
    });
    vector.push(SimpleStruct {
        x: number4, y: b4
    });

    assert(vector.len() == 5);
    assert(vector.capacity() == 8);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val.x == number0);
            assert(val.y == b0);
        },
        Option::None => revert(0), 
    }

    // Push after get
    vector.push(SimpleStruct {
        x: number5, y: b5
    });
    vector.push("this should break it 6");
    vector.push(SimpleStruct {
        x: number6, y: b6
    });
    vector.push(SimpleStruct {
        x: number7, y: b7
    });
    vector.push(SimpleStruct {
        x: number8, y: b8
    });

    match vector.get(4) {
        Option::Some(val) => {
            assert(val.x == number4);
            assert(val.y == b4);
        },
        Option::None => revert(0), 
    }

    match vector.get(6) {
        Option::Some(val) => {
            assert(val.x == number6);
            assert(val.y == b6);
        },
        Option::None => revert(0), 
    }

    assert(vector.len() == 9);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    // Test after capacity change
    match vector.get(4) {
        Option::Some(val) => {
            assert(val.x == number4);
            assert(val.y == b4);
        },
        Option::None => revert(0), 
    }

    match vector.get(6) {
        Option::Some(val) => {
            assert(val.x == number6);
            assert(val.y == b6);
        },
        Option::None => revert(0), 
    }

    vector.clear();

    // Empty after clear
    assert(vector.len() == 0);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == true);

    match vector.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    vector.push(SimpleStruct {
        x: number0, y: b0
    });
    vector.push(SimpleStruct {
        x: number1, y: b1
    });
    vector.push(SimpleStruct {
        x: number2, y: b2
    });
    vector.push(SimpleStruct {
        x: number3, y: b3
    });
    vector.push(SimpleStruct {
        x: number4, y: b4
    });
    vector.push("this should break it 7");

    assert(vector.len() == 5);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    match vector.get(4) {
        Option::Some(val) => {
            assert(val.x == number4);
            assert(val.y == b4);
        },
        Option::None => revert(0), 
    }

    // Out of bounds access
    match vector.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}

fn test_vector_new_enum() {
    let mut vector: Vec<SimpleEnum> = ~Vec::new();

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;

    assert(vector.len() == 0);
    assert(vector.capacity() == 0);
    assert(vector.is_empty() == true);

    vector.push(SimpleEnum::Y(b0));
    vector.push(SimpleEnum::X);
    vector.push(SimpleEnum::Z((b1, b2)));
    vector.push("this should break it 8");

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            match val {
                SimpleEnum::Y(b) => assert(b == b0), _ => revert(0), 
            }
        },
        Option::None => revert(0), 
    }

    match vector.get(1) {
        Option::Some(val) => {
            match val {
                SimpleEnum::X => {
                },
                _ => revert(0), 
            }
        },
        Option::None => revert(0), 
    }

    match vector.get(2) {
        Option::Some(val) => {
            match val {
                SimpleEnum::Z(t) => {
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
    let mut vector: Vec<(u16, b256)> = ~Vec::new();

    let number0 = 0u16;
    let number1 = 1u16;
    let number2 = 2u16;
    let number3 = 3u16;
    let number4 = 4u16;
    let number5 = 5u16;
    let number6 = 6u16;
    let number7 = 7u16;
    let number8 = 8u16;

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;
    let b3 = 0x0000000000000000000000000000000000000000000000000000000000000003;
    let b4 = 0x0000000000000000000000000000000000000000000000000000000000000004;
    let b5 = 0x0000000000000000000000000000000000000000000000000000000000000005;
    let b6 = 0x0000000000000000000000000000000000000000000000000000000000000006;
    let b7 = 0x0000000000000000000000000000000000000000000000000000000000000007;
    let b8 = 0x0000000000000000000000000000000000000000000000000000000000000008;

    assert(vector.len() == 0);
    assert(vector.capacity() == 0);
    assert(vector.is_empty() == true);

    vector.push((number0, b0));
    vector.push((number1, b1));
    vector.push((number2, b2));
    vector.push((number3, b3));
    vector.push((number4, b4));

    assert(vector.len() == 5);
    assert(vector.capacity() == 8);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val.0 == number0);
            assert(val.1 == b0);
        },
        Option::None => revert(0), 
    }

    // Push after get
    vector.push((number5, b5));
    vector.push((number6, b6));
    vector.push((number7, b7));
    vector.push((number8, b8));

    match vector.get(4) {
        Option::Some(val) => {
            assert(val.0 == number4);
            assert(val.1 == b4);
        },
        Option::None => revert(0), 
    }

    match vector.get(6) {
        Option::Some(val) => {
            assert(val.0 == number6);
            assert(val.1 == b6);
        },
        Option::None => revert(0), 
    }

    assert(vector.len() == 9);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    // Test after capacity change
    match vector.get(4) {
        Option::Some(val) => {
            assert(val.0 == number4);
            assert(val.1 == b4);
        },
        Option::None => revert(0), 
    }

    match vector.get(6) {
        Option::Some(val) => {
            assert(val.0 == number6);
            assert(val.1 == b6);
        },
        Option::None => revert(0), 
    }

    vector.clear();

    // Empty after clear
    assert(vector.len() == 0);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == true);

    match vector.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    vector.push((number0, b0));
    vector.push((number1, b1));
    vector.push((number2, b2));
    vector.push((number3, b3));
    vector.push((number4, b4));

    assert(vector.len() == 5);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    match vector.get(4) {
        Option::Some(val) => {
            assert(val.0 == number4);
            assert(val.1 == b4);
        },
        Option::None => revert(0), 
    }

    // Out of bounds access
    match vector.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}

fn test_vector_new_string() {
    let mut vector: Vec<str[4]> = ~Vec::new();

    let s0 = "fuel";
    let s1 = "john";
    let s2 = "nick";

    assert(vector.len() == 0);
    assert(vector.capacity() == 0);
    assert(vector.is_empty() == true);

    vector.push(s0);
    vector.push(s1);
    vector.push(s2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    // Can't compare strings directly. Compare their hashes instead.
    match vector.get(0) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s0));
        },
        Option::None => revert(0), 
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s1));
        },
        Option::None => revert(0), 
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s2));
        },
        Option::None => revert(0), 
    }
}

fn test_vector_new_array() {
    let mut vector: Vec<[u64; 3]> = ~Vec::new();

    let a0 = [0, 1, 2];
    let a1 = [3, 4, 5];
    let a2 = [6, 7, 8];

    assert(vector.len() == 0);
    assert(vector.capacity() == 0);
    assert(vector.is_empty() == true);

    vector.push(a0);
    vector.push(a1);
    vector.push(a2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    // Can't compare strings directly. Compare their hashes instead.
    match vector.get(0) {
        Option::Some(val) => {
            assert(val[0] == a0[0]);
            assert(val[1] == a0[1]);
            assert(val[2] == a0[2]);
        },
        Option::None => revert(0), 
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val[0] == a1[0]);
            assert(val[1] == a1[1]);
            assert(val[2] == a1[2]);
        },
        Option::None => revert(0), 
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val[0] == a2[0]);
            assert(val[1] == a2[1]);
            assert(val[2] == a2[2]);
        },
        Option::None => revert(0), 
    }
}

fn test_vector_with_capacity_u64() {
    let mut vector: Vec<u64> = ~Vec::with_capacity::<u64>(8);

    let number0 = 0;
    let number1 = 1;
    let number2 = 2;
    let number3 = 3;
    let number4 = 4;
    let number5 = 5;
    let number6 = 6;
    let number7 = 7;
    let number8 = 8;

    assert(vector.len() == 0);
    assert(vector.capacity() == 8);
    assert(vector.is_empty() == true);

    vector.push(number0);
    vector.push(number1);
    vector.push(number2);
    vector.push(number3);
    vector.push(number4);

    assert(vector.len() == 5);
    assert(vector.capacity() == 8);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => assert(val == number0), Option::None => revert(0), 
    }

    // Push after get
    vector.push(number5);
    vector.push(number6);
    vector.push(number7);
    vector.push(number8);

    match vector.get(4) {
        Option::Some(val) => assert(val == number4), Option::None => revert(0), 
    }

    match vector.get(6) {
        Option::Some(val) => assert(val == number6), Option::None => revert(0), 
    }

    assert(vector.len() == 9);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    vector.clear();

    // Empty after clear
    assert(vector.len() == 0);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == true);

    match vector.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    vector.push(number0);
    vector.push(number1);
    vector.push(number2);
    vector.push(number3);
    vector.push(number4);

    assert(vector.len() == 5);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    match vector.get(4) {
        Option::Some(val) => assert(val == number4), Option::None => revert(0), 
    }

    // Out of bounds access
    match vector.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}
