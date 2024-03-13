contract;

// ANCHOR: struct_storage_declaration
struct Type1 {
    x: u64,
    y: u64,
}

struct Type2 {
    w: b256,
    z: bool,
}

impl Type2 {
    // a constructor that evaluates to a constant during compilation
    fn default() -> Self {
        Self {
            w: 0x0000000000000000000000000000000000000000000000000000000000000000,
            z: true,
        }
    }
}

storage {
    var1: Type1 = Type1 { x: 0, y: 0 },
    var2: Type2 = Type2::default(),
}
// ANCHOR_END: struct_storage_declaration

abi StorageExample {
    #[storage(write)]
    fn store_struct();

    #[storage(read)]
    fn get_struct();
}

impl StorageExample for Contract {
    #[storage(write)]
    fn store_struct() {
        // ANCHOR: struct_storage_write
        // Store individual fields
        storage.var1.x.write(42);
        storage.var1.y.write(77);

        // Store an entire struct
        let new_struct = Type2 {
            w: 0x1111111111111111111111111111111111111111111111111111111111111111,
            z: false,
        };
        storage.var2.write(new_struct);
        // ANCHOR_END: struct_storage_write
    }

    #[storage(read)]
    fn get_struct() {
        // ANCHOR: struct_storage_read
        let var1_x: u64 = storage.var1.x.try_read().unwrap_or(0);
        let var1_y: u64 = storage.var1.y.try_read().unwrap_or(0);
        let var2: Type2 = storage.var2.try_read().unwrap_or(Type2::default());
        // ANCHOR_END: struct_storage_read
    }
}
