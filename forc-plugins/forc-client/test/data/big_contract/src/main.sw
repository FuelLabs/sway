contract;

use std::storage::storage_vec::*;
use std::hash::*;

abi MyContract {
    fn large_blob() -> bool;
    
    fn enum_input_output(loc: Location) -> Location;
    
    fn struct_input_output(person: Person) -> Person;
    
    #[storage(read, write)]
    fn push_storage(value: u16);

    #[storage(read)]
    fn get_storage(index: u64) -> u16;

    fn assert_configurables() -> bool;
}

enum Location {
    Earth: u64,
    Mars: (),
}

struct Person {
    name: str,
    age: u64,
    alive: bool,
    location: Location,
    some_tuple: (bool, u64),
    some_array: [u64; 2],
    some_b256: b256,
}

struct SimpleStruct {
    a: bool,
    b: u64,
}

storage {
    my_vec: StorageVec<u16> = StorageVec {},
}

configurable {
    BOOL: bool = true,
    U8: u8 = 1,
    U16: u16 = 2,
    U32: u32 = 3,
    U64: u32 = 4,
    U256: u256 = 0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAu256,
    B256: b256 = 0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB,
    CONFIGURABLE_STRUCT: SimpleStruct = SimpleStruct { a: true, b: 5 },
    CONFIGURABLE_ENUM: Location = Location::Earth(1),
    ARRAY_BOOL: [bool; 3] = [true, false, true],
    ARRAY_U64: [u64; 3] = [9, 8, 7],
    TUPLE_BOOL_U64: (bool, u64) = (true, 11),
    STR_4: str[4] = __to_str_array("abcd"),
}

impl core::ops::Eq for Location {
    fn eq(self, other: Location) -> bool {
        match (self, other) {
            (Location::Earth(inner1), Location::Earth(inner2)) => inner1 == inner2,
            (Location::Mars, Location::Mars) => true,
            _ => false,
        }
    }
}

impl MyContract for Contract {
    fn large_blob() -> bool {
        asm() {
            blob i91000;
        }
        true
    }

    fn enum_input_output(loc: Location) -> Location {
        loc
    }

    fn struct_input_output(person: Person) -> Person {
        person
    }

    #[storage(read, write)]
    fn push_storage(value: u16) {
        storage.my_vec.push(value);
    }

    #[storage(read)]
    fn get_storage(index: u64) -> u16 {
        storage.my_vec.get(index).unwrap().read()
    }

    fn assert_configurables() -> bool {
        assert(BOOL == true);
        assert(U8 == 1);
        assert(U16 == 2);
        assert(U32 == 3);
        assert(U64 == 4);
        assert(U256 == 0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAu256);
        assert(B256 == 0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB);
        assert(CONFIGURABLE_STRUCT.a == true);
        assert(CONFIGURABLE_STRUCT.b == 5);
        assert(CONFIGURABLE_ENUM == Location::Earth(1));
        assert(ARRAY_BOOL[0] == true);
        assert(ARRAY_BOOL[1] == false);
        assert(ARRAY_BOOL[2] == true);
        assert(ARRAY_U64[0] == 9);
        assert(ARRAY_U64[1] == 8);
        assert(ARRAY_U64[2] == 7);
        assert(TUPLE_BOOL_U64.0 == true);
        assert(TUPLE_BOOL_U64.1 == 11);
        assert(sha256_str_array(STR_4) == sha256("abcd"));

        // Assert address do not change
        let addr_1 = asm(addr: __addr_of(&BOOL)) {
            addr: u64
        };
        let addr_2 = asm(addr: __addr_of(&BOOL)) {
            addr: u64
        };
        assert(addr_1 == addr_2);
        true
    }
}
