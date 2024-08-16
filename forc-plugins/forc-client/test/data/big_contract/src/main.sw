contract;

abi MyContract {
    fn large_blob() -> bool;
    fn enum_input_output(loc: Location) -> Location;
    fn struct_input_output(person: Person) -> Person;
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

impl MyContract for Contract {
    fn large_blob() -> bool {
        asm() {
            blob i9100;
        }
        true
    }

    fn enum_input_output(loc: Location) -> Location {
        loc
    }

    fn struct_input_output(person: Person) -> Person {
        person
    }
}
