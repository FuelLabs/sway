contract;

use std::{context::registers::stack_ptr, storage::{get, store}};

pub enum MediumEnum {
    One: u64,
    Two: bool,
    Three: b256,
}

pub struct SmallStruct {
    x: u64,
}

pub struct MediumStruct {
    x: u64,
    y: u32,
}

pub struct LargeStruct {
    x: u32,
    y: b256,
    z: u32,
}

pub struct VeryLargeStruct {
    x: u32,
    y: b256,
    z: b256,
}

pub enum StorageEnum {
    V1: b256,
    V2: u64,
    V3: b256,
}

// Storage delimiters
const S_1: b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
const S_2: b256 = 0x0000000000000000000000000000000000000000000000000000000000000002;
const S_3: b256 = 0x0000000000000000000000000000000000000000000000000000000000000003;
const S_4: b256 = 0x0000000000000000000000000000000000000000000000000000000000000004;
const S_5: b256 = 0x0000000000000000000000000000000000000000000000000000000000000005;
const S_6: b256 = 0x0000000000000000000000000000000000000000000000000000000000000006;
const S_7: b256 = 0x0000000000000000000000000000000000000000000000000000000000000007;
const S_8: b256 = 0x0000000000000000000000000000000000000000000000000000000000000008;
const S_9: b256 = 0x0000000000000000000000000000000000000000000000000000000000000009;
const S_10: b256 = 0x0000000000000000000000000000000000000000000000000000000000000010;
const S_11: b256 = 0x0000000000000000000000000000000000000000000000000000000000000011;
const S_12: b256 = 0x0000000000000000000000000000000000000000000000000000000000000012;
const S_13: b256 = 0x0000000000000000000000000000000000000000000000000000000000000013;
const S_14: b256 = 0x0000000000000000000000000000000000000000000000000000000000000014;
const S_15: b256 = 0x0000000000000000000000000000000000000000000000000000000000000015;

abi StorageTest {
    #[storage(write)]fn store_bool(value: bool);
    #[storage(read)]fn get_bool() -> bool;
    #[storage(write)]fn store_u8(value: u8);
    #[storage(read)]fn get_u8() -> u8;
    #[storage(write)]fn store_u16(value: u16);
    #[storage(read)]fn get_u16() -> u16;
    #[storage(write)]fn store_u32(value: u32);
    #[storage(read)]fn get_u32() -> u32;
    #[storage(write)]fn store_u64(value: u64);
    #[storage(read)]fn get_u64() -> u64;
    #[storage(write)]fn store_b256(value: b256);
    #[storage(read)]fn get_b256() -> b256;

    #[storage(write)]fn store_small_struct(value: SmallStruct);
    #[storage(read)]fn get_small_struct() -> SmallStruct;
    #[storage(write)]fn store_medium_struct(value: MediumStruct);
    #[storage(read)]fn get_medium_struct() -> MediumStruct;
    #[storage(write)]fn store_large_struct(value: LargeStruct);
    #[storage(read)]fn get_large_struct() -> LargeStruct;
    #[storage(write)]fn store_very_large_struct(value: VeryLargeStruct);
    #[storage(read)]fn get_very_large_struct() -> VeryLargeStruct;

    #[storage(write)]fn store_enum(value: StorageEnum);
    #[storage(read)]fn get_enum() -> StorageEnum;

    #[storage(write)]fn store_tuple(value: (b256, u8, b256));
    #[storage(read)]fn get_tuple() -> (b256, u8, b256);

    #[storage(write)]fn store_string(value: str[31]);
    #[storage(read)]fn get_string() -> str[31];

    #[storage(write)]fn store_array();
    #[storage(read)]fn get_array() -> [b256;
    3];

    #[storage(read, write)]
    fn storage_in_call() -> u64;
}

impl StorageTest for Contract {
    #[storage(write)]fn store_bool(value: bool) {
        store(S_1, value);
    }

    #[storage(read)]fn get_bool() -> bool {
        get(S_1)
    }

    #[storage(write)]fn store_u8(value: u8) {
        store(S_2, value);
    }

    #[storage(read)]fn get_u8() -> u8 {
        get(S_2)
    }

    #[storage(write)]fn store_u16(value: u16) {
        store(S_3, value);
    }

    #[storage(read)]fn get_u16() -> u16 {
        get(S_3)
    }

    #[storage(write)]fn store_u32(value: u32) {
        store(S_4, value);
    }

    #[storage(read)]fn get_u32() -> u32 {
        get(S_4)
    }

    #[storage(write)]fn store_u64(value: u64) {
        store(S_5, value);
    }

    #[storage(read)]fn get_u64() -> u64 {
        get(S_5)
    }

    #[storage(write)]fn store_b256(value: b256) {
        store(S_6, value);
    }

    #[storage(read)]fn get_b256() -> b256 {
        get(S_6)
    }

    #[storage(write)]fn store_small_struct(value: SmallStruct) {
        store(S_8, value);
    }

    #[storage(read)]fn get_small_struct() -> SmallStruct {
        get(S_8)
    }

    #[storage(write)]fn store_medium_struct(value: MediumStruct) {
        store(S_9, value);
    }

    #[storage(read)]fn get_medium_struct() -> MediumStruct {
        get(S_9)
    }

    #[storage(write)]fn store_large_struct(value: LargeStruct) {
        store(S_9, value);
    }

    #[storage(read)]fn get_large_struct() -> LargeStruct {
        get(S_9)
    }

    #[storage(write)]fn store_very_large_struct(value: VeryLargeStruct) {
        store(S_10, value);
    }

    #[storage(read)]fn get_very_large_struct() -> VeryLargeStruct {
        get(S_10)
    }

    #[storage(write)]fn store_enum(value: StorageEnum) {
        store(S_11, value);
    }

    #[storage(read)]fn get_enum() -> StorageEnum {
        get(S_11)
    }

    #[storage(write)]fn store_tuple(value: (b256, u8, b256)) {
        store(S_12, value);
    }

    #[storage(read)]fn get_tuple() -> (b256, u8, b256) {
        get(S_12)
    }

    #[storage(write)]fn store_string(value: str[31]) {
        store(S_13, value);
    }

    #[storage(read)]fn get_string() -> str[31] {
        get(S_13)
    }

    // Passing arrays into contract methods is not working at the moment
    #[storage(write)]fn store_array() {
        let a = [0x9999999999999999999999999999999999999999999999999999999999999999, 0x8888888888888888888888888888888888888888888888888888888888888888, 0x7777777777777777777777777777777777777777777777777777777777777777];
        store(S_14, a);
    }

    #[storage(read)]fn get_array() -> [b256;
    3] {
        get(S_14)
    }

    #[storage(read, write)]
    fn storage_in_call() -> u64 {
        // The point of this test is to call the storage functions from a non-entry point function,
        // from a function which is _not_ inlined into the entry function.  It then must preserve
        // the stack properly and not leak data structures read or written on the stack, else the
        // function call frame will be corrupt.
        //
        // To avoid inlining the function must be called multiple times and be sufficiently large.
        let pre_sp = stack_ptr();
        let res = non_inlined_function(456_u32) && non_inlined_function(654_u32);
        let post_sp = stack_ptr();

        if pre_sp != post_sp {
            111         // Code to indicate bad stack (it would probably crash before here though).
        } else if !res {
            222         // Code to indicate storage I/O failure.
        } else {
            333         // Code for success - something non-trivial so we can't accidentally succeed.
        }
    }
}

#[storage(read, write)]
fn non_inlined_function(arg: u32) -> bool {
    // By storing and reading from a large complex data structure we're ensuring that this function
    // is too large to be inlined.  The stored value type must be a reference type too, to ensure
    // the use of memory (not a register) to read it back.
    store(S_15, LargeStruct {
        x: arg,
        y: 0x9999999999999999999999999999999999999999999999999999999999999999,
        z: arg,
    });

    let ls: LargeStruct = get(S_15);
    ls.x == arg
}
