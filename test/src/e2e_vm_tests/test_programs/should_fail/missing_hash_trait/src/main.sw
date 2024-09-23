contract;

struct MyStruct {
    val: u64
}

abi MyContract {
    #[storage(read, write)]
    fn test_function();
}

storage {
    my_map: StorageMap<MyStruct, u64> = StorageMap::<MyStruct, u64> {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn test_function() {
        let my_struct = MyStruct { val: 1 };
        storage.my_map.insert(my_struct, 2)
    }
}