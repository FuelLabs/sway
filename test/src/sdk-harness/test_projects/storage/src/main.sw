contract;

use std::registers::stack_ptr;
use std::storage::storage_api::*;

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
    #[storage(read, write)]
    fn store_bool(value: bool);
    #[storage(read)]
    fn get_bool() -> Option<bool>;
    #[storage(read, write)]
    fn store_u8(value: u8);
    #[storage(read)]
    fn get_u8() -> Option<u8>;
    #[storage(read, write)]
    fn store_u16(value: u16);
    #[storage(read)]
    fn get_u16() -> Option<u16>;
    #[storage(read, write)]
    fn store_u32(value: u32);
    #[storage(read)]
    fn get_u32() -> Option<u32>;
    #[storage(read, write)]
    fn store_u64(value: u64);
    #[storage(read)]
    fn get_u64() -> Option<u64>;
    #[storage(read, write)]
    fn store_b256(value: b256);
    #[storage(read)]
    fn get_b256() -> Option<b256>;

    #[storage(read, write)]
    fn store_small_struct(value: SmallStruct);
    #[storage(read)]
    fn get_small_struct() -> Option<SmallStruct>;
    #[storage(read, write)]
    fn store_medium_struct(value: MediumStruct);
    #[storage(read)]
    fn get_medium_struct() -> Option<MediumStruct>;
    #[storage(read, write)]
    fn store_large_struct(value: LargeStruct);
    #[storage(read)]
    fn get_large_struct() -> Option<LargeStruct>;
    #[storage(read, write)]
    fn store_very_large_struct(value: VeryLargeStruct);
    #[storage(read)]
    fn get_very_large_struct() -> Option<VeryLargeStruct>;

    #[storage(read, write)]
    fn store_enum(value: StorageEnum);
    #[storage(read)]
    fn get_enum() -> Option<StorageEnum>;

    #[storage(read, write)]
    fn store_tuple(value: (b256, u8, b256));
    #[storage(read)]
    fn get_tuple() -> Option<(b256, u8, b256)>;

    #[storage(read, write)]
    fn store_string(value: str[31]);
    #[storage(read)]
    fn get_string() -> Option<str[31]>;

    #[storage(read, write)]
    fn store_array();
    #[storage(read)]
    fn get_array() -> Option<[b256; 3]>;

    #[storage(read, write)]
    fn storage_in_call() -> u64;
}

impl StorageTest for Contract {
    #[storage(read, write)]
    fn store_bool(value: bool) {
        write(S_1, 0, value);
    }

    #[storage(read)]
    fn get_bool() -> Option<bool> {
        read::<bool>(S_1, 0)
    }

    #[storage(read, write)]
    fn store_u8(value: u8) {
        write(S_2, 0, value);
    }

    #[storage(read)]
    fn get_u8() -> Option<u8> {
        read::<u8>(S_2, 0)
    }

    #[storage(read, write)]
    fn store_u16(value: u16) {
        write(S_3, 0, value);
    }

    #[storage(read)]
    fn get_u16() -> Option<u16> {
        read::<u16>(S_3, 0)
    }

    #[storage(read, write)]
    fn store_u32(value: u32) {
        write(S_4, 0, value);
    }

    #[storage(read)]
    fn get_u32() -> Option<u32> {
        read::<u32>(S_4, 0)
    }

    #[storage(read, write)]
    fn store_u64(value: u64) {
        write(S_5, 0, value);
    }

    #[storage(read)]
    fn get_u64() -> Option<u64> {
        read::<u64>(S_5, 0)
    }

    #[storage(read, write)]
    fn store_b256(value: b256) {
        write(S_6, 0, value);
    }

    #[storage(read)]
    fn get_b256() -> Option<b256> {
        read::<b256>(S_6, 0)
    }

    #[storage(read, write)]
    fn store_small_struct(value: SmallStruct) {
        write(S_7, 0, value);
    }

    #[storage(read)]
    fn get_small_struct() -> Option<SmallStruct> {
        read::<SmallStruct>(S_7, 0)
    }

    #[storage(read, write)]
    fn store_medium_struct(value: MediumStruct) {
        write(S_8, 0, value);
    }

    #[storage(read)]
    fn get_medium_struct() -> Option<MediumStruct> {
        read::<MediumStruct>(S_8, 0)
    }

    #[storage(read, write)]
    fn store_large_struct(value: LargeStruct) {
        write(S_9, 0, value);
    }

    #[storage(read)]
    fn get_large_struct() -> Option<LargeStruct> {
        read::<LargeStruct>(S_9, 0)
    }

    #[storage(read, write)]
    fn store_very_large_struct(value: VeryLargeStruct) {
        write(S_10, 0, value);
    }

    #[storage(read)]
    fn get_very_large_struct() -> Option<VeryLargeStruct> {
        read::<VeryLargeStruct>(S_10, 0)
    }

    #[storage(read, write)]
    fn store_enum(value: StorageEnum) {
        write(S_11, 0, value);
    }

    #[storage(read)]
    fn get_enum() -> Option<StorageEnum> {
        read::<StorageEnum>(S_11, 0)
    }

    #[storage(read, write)]
    fn store_tuple(value: (b256, u8, b256)) {
        write(S_12, 0, value);
    }

    #[storage(read)]
    fn get_tuple() -> Option<(b256, u8, b256)> {
        read::<(b256, u8, b256)>(S_12, 0)
    }

    #[storage(read, write)]
    fn store_string(value: str[31]) {
        write(S_13, 0, value);
    }

    #[storage(read)]
    fn get_string() -> Option<str[31]> {
        read::<str[31]>(S_13, 0)
    }

    #[storage(read, write)]
    fn store_array() {
        let a = [
            0x9999999999999999999999999999999999999999999999999999999999999999,
            0x8888888888888888888888888888888888888888888888888888888888888888,
            0x7777777777777777777777777777777777777777777777777777777777777777,
        ];
        write(S_14, 0, a);
    }

    #[storage(read)]
    fn get_array() -> Option<[b256; 3]> {
        read::<[b256; 3]>(S_14, 0)
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
            111 // Code to indicate bad stack (it would probably crash before here though).
        } else if !res {
            222 // Code to indicate storage I/O failure.
        } else {
            333 // Code for success - something non-trivial so we can't accidentally succeed.
        }
    }
}

#[storage(read, write)]
fn non_inlined_function(arg: u32) -> bool {
    // By storing and reading from a large complex data structure we're ensuring that this function
    // is too large to be inlined.  The stored value type must be a reference type too, to ensure
    // the use of memory (not a register) to read it back.
    write(S_15, 0, LargeStruct {
        x: arg,
        y: 0x9999999999999999999999999999999999999999999999999999999999999999,
        z: arg,
    });

    let ls = read::<LargeStruct>(S_15, 0).unwrap();
    ls.x == arg
}
