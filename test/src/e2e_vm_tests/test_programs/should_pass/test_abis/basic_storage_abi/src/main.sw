library basic_storage_abi;

pub struct Quad {
    v1: u64,
    v2: u64,
    v3: u64,
    v4: u64,
}

abi StoreU64 {
    #[storage(write)]
    fn store_u64(key: b256, value: u64);
    #[storage(read)]
    fn get_u64(key: b256) -> u64;

    #[storage(write)]
    fn intrinsic_store_word(key: b256, value: u64);
    #[storage(read)]
    fn intrinsic_load_word(key: b256) -> u64;

    #[storage(write)]
    fn intrinsic_store_quad(key: b256, value: Quad);
    #[storage(read)]
    fn intrinsic_load_quad(key: b256) -> Quad;

    #[storage(read, write)]
    fn test_storage_exhaustive();
}
