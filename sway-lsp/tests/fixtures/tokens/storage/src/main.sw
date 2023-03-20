contract;

struct Type1 {
    x: u64,
    y: bool,
    z: Type2,
}

struct Type2 {
    x: u64,
}

storage {
    var1: Type1 = Type1 { x:0, y: false, z: Type2 { x:0 } },
}

abi StorageExample {
    #[storage(write)]
    fn store_something();
}

impl StorageExample  for Contract {
    #[storage(write)]
    fn store_something() {
        storage.var1.x = 42;
        storage.var1.y = true;
        storage.var1.z.x = 1337;
    }
}
