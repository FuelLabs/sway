contract;

dep storage_lib;

use storage_in_library_abi::StorageInLibrary as SomethingElse;

storage {
    some_var: bool = false,
    contract_foo: u64 = 0,
    some_other_var: u64 = 0,
}

use storage_lib::storage.foo as storage.contract_foo;

impl SomethingElse for Contract {
    #[storage(read, write)]
    fn call_update_library_storage() {
        // Update from library
        storage_lib::update_library_storage();

        // Local update
        storage.contract_foo += 1;
    }

    #[storage(read)]
    fn call_get_library_storage() -> u64 {
        storage_lib::get_library_storage()
    }
}
