contract;

// ANCHOR: basic_storage_declaration
storage {
    var1: u64 = 1,
    var2: b256 = b256::zero(),
    var3: Address = Address::zero(),
    var4: Option<u8> = None,
}
// ANCHOR_END: basic_storage_declaration

abi StorageExample {
    #[storage(write)]
    fn store_something();

    #[storage(read)]
    fn get_something();
}

impl StorageExample for Contract {
    #[storage(write)]
    fn store_something() {
        // ANCHOR: basic_storage_write
        storage.var1.write(42);
        storage
            .var2
            .write(0x1111111111111111111111111111111111111111111111111111111111111111);
        storage
            .var3
            .write(Address::from(0x1111111111111111111111111111111111111111111111111111111111111111));
        storage.var4.write(Some(2u8));
        // ANCHOR_END: basic_storage_write
    }
    #[storage(read)]
    fn get_something() {
        // ANCHOR: basic_storage_read
        let var1: u64 = storage.var1.read();
        let var2: b256 = storage.var2.try_read().unwrap_or(b256::zero());
        let var3: Address = storage.var3.try_read().unwrap_or(Address::zero());
        let var4: Option<u8> = storage.var4.try_read().unwrap_or(None);
        // ANCHOR_END: basic_storage_read
    }
}
