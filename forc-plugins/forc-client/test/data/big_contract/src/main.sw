contract;

use std::storage::storage_vec::*;
use std::hash::*;

abi MyContract {
    fn large_blob() -> bool;
    
    fn enum_input_output(loc: Location) -> Location;
    
    fn struct_input_output(person: Person) -> Person;
    
    fn array_of_enum_input_output(aoe: [Location; 2]) -> [Location; 2];
    
    #[storage(read, write)]
    fn push_storage_u16(value: u16);

    #[storage(read)]
    fn get_storage_u16(index: u64) -> u16;

    #[storage(read, write)]
    fn push_storage_simple(value: SimpleStruct);

    #[storage(read)]
    fn get_storage_simple(index: u64) -> SimpleStruct;

    #[storage(read, write)]
    fn push_storage_location(value: Location);

    #[storage(read)]
    fn get_storage_location(index: u64) -> Location;

    fn assert_configurables() -> bool;
}

enum Location {
    Earth: u64,
    Mars: (),
    SimpleJupiter: Color,
    Jupiter: [Color; 2],
    SimplePluto: SimpleStruct,
    Pluto: [SimpleStruct; 2],
}

enum Color {
    Red: (),
    Blue: u64,
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
    my_simple_vec: StorageVec<SimpleStruct> = StorageVec {},
    my_location_vec: StorageVec<Location> = StorageVec {},
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
    ARRAY_LOCATION: [Location; 2] = [Location::Earth(10), Location::Mars],
    ARRAY_SIMPLE_STRUCT: [SimpleStruct; 3] = [ SimpleStruct { a: true, b: 5}, SimpleStruct { a: false, b: 0 }, SimpleStruct { a: true, b: u64::max() }],
    TUPLE_BOOL_U64: (bool, u64) = (true, 11),
    STR_4: str[4] = __to_str_array("abcd"),
}

impl PartialEq for Color {
    fn eq(self, other: Color) -> bool {
        match (self, other) {
            (Color::Red, Color::Red) => true,
            (Color::Blue(inner1), Color::Blue(inner2)) => inner1 == inner2,
            _ => false,
        }
    }
}

impl PartialEq for SimpleStruct {
    fn eq(self, other: SimpleStruct) -> bool {
        self.a == other.a && self.b == other.b
    }
}

impl PartialEq for Location {
    fn eq(self, other: Location) -> bool {
        match (self, other) {
            (Location::Earth(inner1), Location::Earth(inner2)) => inner1 == inner2,
            (Location::Mars, Location::Mars) => true,
            (Location::SimpleJupiter(inner1), Location::SimpleJupiter(inner2)) => inner1 == inner2,
            (Location::Jupiter(inner1), Location::Jupiter(inner2)) => (inner1[0] == inner2[0] && inner1[1] == inner2[1]),
            (Location::SimplePluto(inner1), Location::SimplePluto(inner2)) => inner1 == inner2,
            (Location::Pluto(inner1), Location::Pluto(inner2)) => (inner1[0] == inner2[0] && inner1[1] == inner2[1]),
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

    fn array_of_enum_input_output(aoe: [Location; 2]) -> [Location; 2] {
        aoe
    }

    #[storage(read, write)]
    fn push_storage_u16(value: u16) {
        storage.my_vec.push(value);
    }

    #[storage(read)]
    fn get_storage_u16(index: u64) -> u16 {
        storage.my_vec.get(index).unwrap().read()
    }

    #[storage(read, write)]
    fn push_storage_simple(value: SimpleStruct) {
        storage.my_simple_vec.push(value);
    }

    #[storage(read)]
    fn get_storage_simple(index: u64) -> SimpleStruct {
        storage.my_simple_vec.get(index).unwrap().read()
    }
    
    #[storage(read, write)]
    fn push_storage_location(value: Location) {
        storage.my_location_vec.push(value);
    }

    #[storage(read)]
    fn get_storage_location(index: u64) -> Location {
        storage.my_location_vec.get(index).unwrap().read()
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
        assert(ARRAY_LOCATION[0] == Location::Earth(10));
        assert(ARRAY_LOCATION[1] == Location::Mars);
        assert(ARRAY_SIMPLE_STRUCT[0].a == true);
        assert(ARRAY_SIMPLE_STRUCT[0].b == 5);
        assert(ARRAY_SIMPLE_STRUCT[1].a == false);
        assert(ARRAY_SIMPLE_STRUCT[1].b == 0);
        assert(ARRAY_SIMPLE_STRUCT[2].a == true);
        assert(ARRAY_SIMPLE_STRUCT[2].b == u64::max());
        assert(ARRAY_LOCATION[1] == Location::Mars);
        assert(ARRAY_LOCATION[1] == Location::Mars);
        assert(TUPLE_BOOL_U64.0 == true);
        assert(TUPLE_BOOL_U64.1 == 11);
        assert(sha256_str_array(STR_4) == sha256("abcd"));

        // Assert address do not change
        let addr_1 = __transmute::<&bool, u64>(&BOOL);
        let addr_2 = __transmute::<&bool, u64>(&BOOL);
        assert(addr_1 == addr_2);
        true
    }
}
