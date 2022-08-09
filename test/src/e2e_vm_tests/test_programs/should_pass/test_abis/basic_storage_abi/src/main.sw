library basic_storage_abi;

abi StoreU64 {
    #[storage(write)]
    fn store_u64(key: b256, value: u64);
    #[storage(read)]
    fn get_u64(key: b256) -> u64;

    #[storage(write)]
    fn intrinsic_store_word(key: b256, value: u64);
    #[storage(read)]
    fn intrinsic_load_word(key: b256) -> u64;
}
