script;

use std::{assert::assert, hash::sha256, option::Option, revert::revert, u128::{From, U128}, vec::Vec};

enum SimpleEnum {
    A: b256,
    B: (),
}

struct SimpleStruct {
    x: u32,
    y: b256,
}

fn main() -> bool {
    test_vector_swap_u8();
    test_vector_swap_b256();
    test_vector_swap_struct();
    test_vector_swap_enum();
    test_vector_swap_tuple();
    test_vector_swap_string();
    test_vector_swap_array();
    test_vector_swap_same_indexes_noop();
    true
}

fn test_vector_swap_u8() {
    let mut vector = ~Vec::new();

    let number0 = 0u8;
    let number1 = 1u8;
    let number2 = 2u8;

    vector.push(number0);
    vector.push(number1);
    vector.push(number2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val == number0)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val == number1)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val == number2)
        },
        Option::None => {
            revert(0)
        },
    }

    vector.swap(0, 2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val == number2)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val == number1)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val == number0)
        },
        Option::None => {
            revert(0)
        },
    }
}

fn test_vector_swap_b256() {
    let mut vector = ~Vec::new();

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;

    vector.push(b0);
    vector.push(b1);
    vector.push(b2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val == b0)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val == b1)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val == b2)
        },
        Option::None => {
            revert(0)
        },
    }

    vector.swap(0, 2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val == b2)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val == b1)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val == b0)
        },
        Option::None => {
            revert(0)
        },
    }
}

fn test_vector_swap_struct() {
    let mut vector = ~Vec::new();

    let number0 = 0u32;
    let number1 = 1u32;
    let number2 = 2u32;

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;

    vector.push(SimpleStruct {
        x: number0, y: b0
    });
    vector.push(SimpleStruct {
        x: number1, y: b1
    });
    vector.push(SimpleStruct {
        x: number2, y: b2
    });

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val.x == number0);
            assert(val.y == b0);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val.x == number1);
            assert(val.y == b1);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val.x == number2);
            assert(val.y == b2);
        },
        Option::None => {
            revert(0)
        },
    }

    vector.swap(0, 2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val.x == number2);
            assert(val.y == b2);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val.x == number1);
            assert(val.y == b1);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val.x == number0);
            assert(val.y == b0);
        },
        Option::None => {
            revert(0)
        },
    }
}

fn test_vector_swap_enum() {
    let mut vector = ~Vec::new();

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;

    vector.push(SimpleEnum::A(b0));
    vector.push(SimpleEnum::A(b1));
    vector.push(SimpleEnum::B);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            match val {
                SimpleEnum::A(b) => {
                    assert(b == b0)
                },
                _ => {
                    revert(0)
                },
            }
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            match val {
                SimpleEnum::A(b) => {
                    assert(b == b1)
                },
                _ => {
                    revert(0)
                },
            }
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            match val {
                SimpleEnum::B => {
                },
                _ => {
                    revert(0)
                },
            }
        },
        Option::None => {
            revert(0)
        },
    }

    vector.swap(0, 2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            match val {
                SimpleEnum::B => {
                },
                _ => {
                    revert(0)
                },
            }
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            match val {
                SimpleEnum::A(b) => {
                    assert(b == b1)
                },
                _ => {
                    revert(0)
                },
            }
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            match val {
                SimpleEnum::A(b) => {
                    assert(b == b0)
                },
                _ => {
                    revert(0)
                },
            }
        },
        Option::None => {
            revert(0)
        },
    }
}

fn test_vector_swap_tuple() {
    let mut vector = ~Vec::new();

    let number0 = 0u16;
    let number1 = 1u16;
    let number2 = 2u16;

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;

    vector.push((number0, b0));
    vector.push((number1, b1));
    vector.push((number2, b2));

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val.0 == number0);
            assert(val.1 == b0);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val.0 == number1);
            assert(val.1 == b1);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val.0 == number2);
            assert(val.1 == b2);
        },
        Option::None => {
            revert(0)
        },
    }

    vector.swap(0, 2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val.0 == number2);
            assert(val.1 == b2);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val.0 == number1);
            assert(val.1 == b1);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val.0 == number0);
            assert(val.1 == b0);
        },
        Option::None => {
            revert(0)
        },
    }
}

fn test_vector_swap_string() {
    let mut vector = ~Vec::new();

    let s0 = "fuel";
    let s1 = "john";
    let s2 = "nick";

    vector.push(s0);
    vector.push(s1);
    vector.push(s2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s0));
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s1));
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s2));
        },
        Option::None => {
            revert(0)
        },
    }

    vector.swap(0, 2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s2));
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s1));
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(sha256(val) == sha256(s0));
        },
        Option::None => {
            revert(0)
        },
    }
}

fn test_vector_swap_array() {
    let mut vector = ~Vec::new();

    let a0 = [0, 1, 2];
    let a1 = [3, 4, 5];
    let a2 = [6, 7, 8];

    vector.push(a0);
    vector.push(a1);
    vector.push(a2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val[0] == a0[0]);
            assert(val[1] == a0[1]);
            assert(val[2] == a0[2]);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val[0] == a1[0]);
            assert(val[1] == a1[1]);
            assert(val[2] == a1[2]);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val[0] == a2[0]);
            assert(val[1] == a2[1]);
            assert(val[2] == a2[2]);
        },
        Option::None => {
            revert(0)
        },
    }

    vector.swap(0, 2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val[0] == a2[0]);
            assert(val[1] == a2[1]);
            assert(val[2] == a2[2]);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val[0] == a1[0]);
            assert(val[1] == a1[1]);
            assert(val[2] == a1[2]);
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val[0] == a0[0]);
            assert(val[1] == a0[1]);
            assert(val[2] == a0[2]);
        },
        Option::None => {
            revert(0)
        },
    }
}

fn test_vector_swap_same_indexes_noop() {
    let mut vector = ~Vec::new();

    let number0 = 0u8;
    let number1 = 1u8;
    let number2 = 2u8;

    vector.push(number0);
    vector.push(number1);
    vector.push(number2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val == number0)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val == number1)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val == number2)
        },
        Option::None => {
            revert(0)
        },
    }

    vector.swap(1, 1);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => {
            assert(val == number0)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(1) {
        Option::Some(val) => {
            assert(val == number1)
        },
        Option::None => {
            revert(0)
        },
    }

    match vector.get(2) {
        Option::Some(val) => {
            assert(val == number2)
        },
        Option::None => {
            revert(0)
        },
    }
}
