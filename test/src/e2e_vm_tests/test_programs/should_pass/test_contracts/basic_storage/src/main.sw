contract;
use std::{storage::*, assert::assert};
use basic_storage_abi::*;

impl StoreU64 for Contract {
    #[storage(read)]
    fn get_u64(storage_key: b256) -> u64 {
        get(storage_key)
    }

    #[storage(write)]
    fn store_u64(key: b256, value: u64) {
        store(key, value);
    }

    #[storage(read)]
    fn intrinsic_load_word(key: b256) -> u64 {
        __state_load_word(key)
    }

    #[storage(write)]
    fn intrinsic_store_word(key: b256, value: u64) {
        __state_store_word(key, value);
    }

    #[storage(read)]
    fn intrinsic_load_quad(key: b256) -> Quad {
       let q = Quad { v1 : 0, v2 : 0, v3 : 0, v4 : 0 };
       let q_addr = __addr_of(q);
        __state_load_quad(key, q_addr);
        q
    }

    #[storage(write)]
    fn intrinsic_store_quad(key: b256, value: Quad) {
       let addr = __addr_of(value);
        __state_store_quad(key, addr)
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

// These inputs are taken from the storage_access_contract test.
#[storage(read, write)]
fn test_storage() {
    let key: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101;

    let x: u64 = 64;
    store(key, x);
    assert(x == get::<u64>(key));

    let y: b256 = 0x1101010101010101010101010101010101010101010101010101010101010101;
    store(key, y);
    assert(y == get::<b256>(key));

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
    store(key, s);
    let s_ = get::<S>(key);
    assert(s.x == s_.x && s.y == s_.y && s.z == s_.z && s.t.x == s_.t.x
           && s.t.y == s_.t.y && s.t.z == s_.t.z && s.t.boolean == s_.t.boolean
           && s.t.int8 == s_.t.int8 && s.t.int16 == s_.t.int16 && s.t.int32 == s_.t.int32);

    let boolean: bool = true;
    store(key, boolean);
    assert(boolean == get::<bool>(key));

    let int8: u8 = 8;
    store(key, int8);
    assert(int8 == get::<u8>(key));
    
    let int16: u16 = 16;
    store(key, int16);
    assert(int16 == get::<u16>(key));

    let int32: u32 = 32;
    store(key, int32);
    assert(int32 == get::<u32>(key));

    let e: E = E::B(T {
        x: 1,
        y: 2,
        z: 0x0000000000000000000000000000000000000000000000000000000000000003,
        boolean: true,
        int8: 4,
        int16: 5,
        int32: 6,
    },
    );
    store(key, e);
    let e_ = get::<E>(key);
    match (e, e_) {
          (E::B(T {x: x1, y: y1, z: z1, boolean: boolean1, int8: int81, int16: int161, int32: int321}),
          E::B(T {x: x2, y: y2, z: z2, boolean: boolean2, int8: int82, int16: int162, int32: int322})) =>
          {
                assert(x1 == x2 && y1 == y2 && z1 == z2 && boolean1 == boolean2 &&
                int81 == int82 && int161 == int162 && int321 == int322);
          }
          _ => assert(false),
    }
    
    let e2: E = E::A(777);
    store(key, e2);
    let e2_ = get::<E>(key);
    match (e2, e2_) {
          (E::A(i1), E::A(i2)) => {
              assert(i1 == i2);
          }
          _ => assert(false),
    }
}
