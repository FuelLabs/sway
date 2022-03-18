library basic_storage_abi;

abi StoreU64 {
    fn store_u64(key: b256, value: u64);
    fn get_u64(key: b256) -> u64;
}
