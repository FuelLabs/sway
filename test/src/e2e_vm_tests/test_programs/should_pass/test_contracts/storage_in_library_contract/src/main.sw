contract;

dep storage_lib;

use storage_in_library_abi::StorageInLibrary as SomethingElse;

storage {
    first: u64 = 0,
    bar: u64 = 0,
}

use storage_lib::storage.foo as storage.bar;

impl SomethingElse for Contract {
    #[storage(write)]
    fn call_library_function() {
        storage_lib::mutate_foo();
        storage.bar = 99;
    }
}
