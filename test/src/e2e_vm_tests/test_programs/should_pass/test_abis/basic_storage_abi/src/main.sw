library;

pub struct Quad {
    pub v1: u64,
    pub v2: u64,
    pub v3: u64,
    pub v4: u64,
}

impl AbiDecode for Quad {
    fn abi_decode(ref mut buffer: BufferReader) -> Quad {
        let v1 = u64::abi_decode(buffer);
        let v2 = u64::abi_decode(buffer);
        let v3 = u64::abi_decode(buffer);
        let v4 = u64::abi_decode(buffer);
        Quad {
            v1,
            v2,
            v3,
            v4
        }
    }
}

abi BasicStorage {
    #[storage(write)]
    fn store_u64(key: b256, value: u64);
    #[storage(read)]
    fn get_u64(key: b256) -> Option<u64>;

    #[storage(write)]
    fn intrinsic_store_word(key: b256, value: u64);
    #[storage(read)]
    fn intrinsic_load_word(key: b256) -> u64;

    #[storage(write)]
    fn intrinsic_store_quad(key: b256, value: Vec<Quad>);
    #[storage(read)]
    fn intrinsic_load_quad(key: b256, slots: u64) -> Vec<Quad>;

    #[storage(read, write)]
    fn test_storage_exhaustive();
}
