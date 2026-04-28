contract;

mod lib;

use lib::*;
use std::storage::storage_api::*;

storage {
    can_init: CanInitStruct = CanInitStruct::init(11, 12),
}

abi WriteAndReadStructWithPrivateFields {
    #[storage(read)]
    fn read_initial_can_init_via_storage() -> CanInitStruct;

    #[storage(read, write)]
    fn write_and_read_can_init_via_storage(input: CanInitStruct) -> CanInitStruct;

    #[storage(read, write)]
    fn write_and_read_cannot_init_via_api(input: CannotInitStruct) -> CannotInitStruct;
}

impl WriteAndReadStructWithPrivateFields for Contract {
    #[storage(read)]
    fn read_initial_can_init_via_storage() -> CanInitStruct {
        storage.can_init.read()
    }

    #[storage(read, write)]
    fn write_and_read_can_init_via_storage(input: CanInitStruct) -> CanInitStruct {
        storage.can_init.write(input);
        let read = storage.can_init.read();
        assert_eq(input, read);

        read
    }

    #[cfg(experimental_dynamic_storage = false)]
    #[storage(read, write)]
    fn write_and_read_cannot_init_via_api(input: CannotInitStruct) -> CannotInitStruct {
        const STORAGE_KEY: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
        write_quads::<CannotInitStruct>(STORAGE_KEY, 0, input);
        let read = read_quads::<CannotInitStruct>(STORAGE_KEY, 0).unwrap();
        assert_eq(input, read);

        read
    }

    #[cfg(experimental_dynamic_storage = true)]
    #[storage(read, write)]
    fn write_and_read_cannot_init_via_api(input: CannotInitStruct) -> CannotInitStruct {
        const STORAGE_KEY: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
        write_slot::<CannotInitStruct>(STORAGE_KEY, input);
        let read = read_slot::<CannotInitStruct>(STORAGE_KEY, 0).unwrap();
        assert_eq(input, read);

        read
    }
}
