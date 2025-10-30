contract;
use std::{hash::*, storage::storage_api::{read, write}};
use basic_storage_abi::*;

const C1 = 1;
const NS1_NS2_C1 = 2;
const S5 = __to_str_array("aaaaa");

const C1KEY: b256 = 0x933a534d4af4c376b0b569e8d8a2c62e635e26f403e124cb91d9c42e83d54373;

storage {
    c1 in C1KEY: u8 = C1,
    str0: str[0] = __to_str_array(""),
    str1: str[1] = __to_str_array("a"),
    str2: str[2] = __to_str_array("aa"),
    str3: str[3] = __to_str_array("aaa"),
    str4: str[4] = __to_str_array("aaaa"),
    str5: str[5] = S5,
    str6: str[6] = __to_str_array("aaaaaa"),
    str7: str[7] = __to_str_array("aaaaaaa"),
    str8: str[8] = __to_str_array("aaaaaaaa"),
    str9: str[9] = __to_str_array("aaaaaaaaa"),
    str10: str[10] = __to_str_array("aaaaaaaaaa"),

    const_u256: u256 = 0x0000000000000000000000000000000000000000000000000000000001234567u256,
    const_b256: b256 = 0x0000000000000000000000000000000000000000000000000000000001234567,
    ns1 {
        ns2 {
            c1: u64 = NS1_NS2_C1,
        }
    }
}

impl BasicStorage for Contract {
    #[storage(read)]
    fn get_u64(storage_key: b256) -> Option<u64> {
        read(storage_key, 0)
    }

    #[storage(write)]
    fn store_u64(key: b256, value: u64) {
        write(key, 0, value);
    }

    #[storage(read)]
    fn intrinsic_load_word(key: b256) -> u64 {
        __state_load_word(key)
    }

    #[storage(write)]
    fn intrinsic_store_word(key: b256, value: u64) {
        let _ = __state_store_word(key, value);
    }

    #[storage(read)]
    fn intrinsic_load_quad(key: b256, slots: u64) -> Vec<Quad> {
        let q = Quad {
            v1: 0,
            v2: 0,
            v3: 0,
            v4: 0,
        };
        let mut values: Vec<Quad> = Vec::new();
        let mut i = 0;
        while i < slots {
            values.push(q);
            i += 1;
        }

        let _ = __state_load_quad(key, values.ptr(), slots);
        values
    }

    #[storage(write)]
    fn intrinsic_store_quad(key: b256, values: Vec<Quad>) {
        let _ = __state_store_quad(key, values.ptr(), values.len());
    }

    #[storage(read, write)]
    fn test_storage_exhaustive() {
        test_storage();
    }
}

pub struct S {
    x: u64,
    y: u64,
    z: b256,
    t: T,
}

pub struct T {
    x: u64,
    y: u64,
    z: b256,
    boolean: bool,
    int8: u8,
    int16: u16,
    int32: u32,
}

pub enum E {
    A: u64,
    B: T,
}

pub enum F {
    A: u8,
    B: bool,
    C: (),
}

pub enum G {
    A: u8,
    B: u64,
}

// These inputs are taken from the storage_access_contract test.
#[storage(read, write)]
fn test_storage() {
    let key: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101;

    let x: u64 = 64;
    write(key, 0, x);
    assert(x == read::<u64>(key, 0).unwrap());

    let y: b256 = 0x1101010101010101010101010101010101010101010101010101010101010101;
    write(key, 0, y);
    assert(y == read::<b256>(key, 0).unwrap());

    let s: S = S {
        x: 1,
        y: 2,
        z: 0x0000000000000000000000000000000000000000000000000000000000000003,
        t: T {
            x: 4,
            y: 5,
            z: 0x0000000000000000000000000000000000000000000000000000000000000006,
            boolean: true,
            int8: 7,
            int16: 8,
            int32: 9,
        },
    };
    write(key, 0, s);
    let s_ = read::<S>(key, 0).unwrap();
    assert(s.x == s_.x && s.y == s_.y && s.z == s_.z);
    assert(s.t.x == s_.t.x && s.t.y == s_.t.y && s.t.z == s_.t.z && s.t.boolean == s_.t.boolean); 
    assert(s.t.int8 == s_.t.int8 && s.t.int16 == s_.t.int16 && s.t.int32 == s_.t.int32);

    let boolean: bool = true;
    write(key, 0, boolean);
    assert(boolean == read::<bool>(key, 0).unwrap());

    let int8: u8 = 8;
    write(key, 0, int8);
    assert(int8 == read::<u8>(key, 0).unwrap());

    let int16: u16 = 16;
    write(key, 0, int16);
    assert(int16 == read::<u16>(key, 0).unwrap());

    let int32: u32 = 32;
    write(key, 0, int32);
    assert(int32 == read::<u32>(key, 0).unwrap());

    let e: E = E::B(T {
        x: 1,
        y: 2,
        z: 0x0000000000000000000000000000000000000000000000000000000000000003,
        boolean: true,
        int8: 4,
        int16: 5,
        int32: 6,
    });
    write(key, 0, e);
    let e_ = read::<E>(key, 0).unwrap();
    match (e, e_) {
        (

            E::B(T {
                x: x1,
                y: y1,
                z: z1,
                boolean: boolean1,
                int8: int81,
                int16: int161,
                int32: int321,
            }),
            E::B(T {
                x: x2,
                y: y2,
                z: z2,
                boolean: boolean2,
                int8: int82,
                int16: int162,
                int32: int322,
            }),
        ) => {
            assert(x1 == x2 && y1 == y2 && z1 == z2 && boolean1 == boolean2);
            assert(int81 == int82 && int161 == int162 && int321 == int322);
        }
        _ => assert(false),
    }

    let e2: E = E::A(777);
    write(key, 0, e2);
    let e2_ = read::<E>(key, 0).unwrap();
    match (e2, e2_) {
        (E::A(i1), E::A(i2)) => {
            assert(i1 == 777);
            assert(i1 == i2);
        }
        _ => assert(false),
    }

    let f1: F = F::A(8);
    write(key, 0, f1);
    let f1_ = read::<F>(key, 0).unwrap();
    match (f1, f1_) {
        (F::A(i1), F::A(i2)) => {
            assert(i1 == 8);
            assert(i1 == i2);
        }
        _ => assert(false),
    }

    let f2: F = F::B(true);
    write(key, 0, f2);
    let f2_ = read::<F>(key, 0).unwrap();
    match (f2, f2_) {
        (F::B(i1), F::B(i2)) => {
            assert(i1 == true);
            assert(i1 == i2);
        }
        _ => assert(false),
    }

    let f3: F = F::C;
    write(key, 0, f3);
    let f3_ = read::<F>(key, 0).unwrap();
    match (f3, f3_) {
        (F::C, F::C) => {
            assert(true);
        }
        _ => assert(false),
    }

    let g1: G = G::A(8);
    write(key, 0, g1);
    let g1_ = read::<G>(key, 0).unwrap();
    match (g1, g1_) {
        (G::A(i1), G::A(i2)) => {
            assert(i1 == 8);
            assert(i1 == i2);
        }
        _ => assert(false),
    }

    let g2: G = G::B(64);
    write(key, 0, g2);
    let g2_ = read::<G>(key, 0).unwrap();
    match (g2, g2_) {
        (G::B(i1), G::B(i2)) => {
            assert(i1 == 64);
            assert(i1 == i2);
        }
        _ => assert(false),
    }

    assert(storage.str0.try_read().is_none());

    assert_streq(storage.str1.read(), "a");
    assert_streq(storage.str2.read(), "aa");
    assert_streq(storage.str3.read(), "aaa");
    assert_streq(storage.str4.read(), "aaaa");
    assert_streq(storage.str5.read(), "aaaaa");
    assert_streq(storage.str6.read(), "aaaaaa");
    assert_streq(storage.str7.read(), "aaaaaaa");
    assert_streq(storage.str8.read(), "aaaaaaaa");
    assert_streq(storage.str9.read(), "aaaaaaaaa");
    assert_streq(storage.str10.read(), "aaaaaaaaaa");

    assert_eq(storage.c1.read(), C1);
    storage.c1.write(2);
    assert_eq(storage.c1.read(), 2);
    
    assert_eq(storage.const_u256.read(), 0x0000000000000000000000000000000000000000000000000000000001234567u256);
    storage.const_u256.write(0x0000000000000000000000000000000000000000000000000000000012345678u256);
    assert_eq(storage.const_u256.read(), 0x0000000000000000000000000000000000000000000000000000000012345678u256);

    assert_eq(storage.const_b256.read(), 0x0000000000000000000000000000000000000000000000000000000001234567);
    storage.const_b256.write(0x0000000000000000000000000000000000000000000000000000000012345678);
    assert_eq(storage.const_b256.read(), 0x0000000000000000000000000000000000000000000000000000000012345678);

    assert_eq(storage::ns1::ns2.c1.read(), NS1_NS2_C1);

    assert_eq(storage.c1.slot(), C1KEY);
}

// If these comparisons are done inline just above then it blows out the register allocator due to
// all the ASM blocks.
#[inline(never)]
fn assert_streq<S1>(lhs: S1, rhs: str) {
    assert(sha256_str_array(lhs) == sha256(rhs));
}

#[test]
fn collect_basic_storage_contract_gas_usages() {
    let caller = abi(BasicStorage, CONTRACT_ID);
    let key = 0x0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let value = 4242;

    caller.store_u64(key, value);
    let _ = caller.get_u64(key);

    let key = 0x00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    caller.intrinsic_store_word(key, value);
    let _ = caller.intrinsic_load_word(key);

    let key = 0x11ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let q = Quad {
        v1: 1,
        v2: 2,
        v3: 4,
        v4: 100,
    };
    let mut values = Vec::new();
    values.push(q);
    caller.intrinsic_store_quad(key, values);
    let _ = caller.intrinsic_load_quad(key, 1).get(0);

    let key = 0x11ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    let q0 = Quad {
        v1: 1,
        v2: 2,
        v3: 4,
        v4: 100,
    };
    let q1 = Quad {
        v1: 2,
        v2: 3,
        v3: 5,
        v4: 101,
    };
    let mut values = Vec::new();
    values.push(q0);
    values.push(q1);
    caller.intrinsic_store_quad(key, values);
    let r = caller.intrinsic_load_quad(key, values.len());
    let r0 = r.get(0);
    let r1 = r.get(1);

    caller.test_storage_exhaustive();
}
