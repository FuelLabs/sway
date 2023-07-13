contract;

// ANCHOR: storage_declaration
struct Type1 {
    x: u64,
    y: u64,
}

struct Type2 {
    w: b256,
    z: bool,
}

storage {
    var1: Type1 = Type1 { x: 0, y: 0 },
    var2: Type2 = Type2 {
        w: 0x0000000000000000000000000000000000000000000000000000000000000000,
        z: false,
    },
}

// ANCHOR_END: storage_declaration
abi StorageExample {
    #[storage(write)]
    fn store_something();

    #[storage(read)]
    fn get_something() -> (u64, u64, b256, bool);
}

impl StorageExample for Contract {
    // ANCHOR: storage_write
    #[storage(write)]
    fn store_something() {
        storage.var1.x.write(42);
        storage.var1.y.write(77);
        storage.var2.w.write(0x1111111111111111111111111111111111111111111111111111111111111111);
        storage.var2.z.write(true);
    }
    // ANCHOR_END: storage_write
    // ANCHOR: storage_read
    #[storage(read)]
    fn get_something() -> (u64, u64, b256, bool) {
        (
            storage.var1.x.try_read().unwrap_or(0),
            storage.var1.y.try_read().unwrap_or(0),
            storage.var2.w.try_read().unwrap_or(0x0000000000000000000000000000000000000000000000000000000000000000),
            storage.var2.z.try_read().unwrap_or(false),
        )
    }
    // ANCHOR_END: storage_read
}
