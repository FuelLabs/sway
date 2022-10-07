library storage_in_library_abi;

abi StorageInLibrary {
    #[storage(read, write)] 
    fn call_update_library_storage();

    #[storage(read)] 
    fn call_get_library_storage() -> u64;
}
