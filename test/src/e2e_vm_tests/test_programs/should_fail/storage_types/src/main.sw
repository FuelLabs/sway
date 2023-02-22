contract;

use std::storage::StorageVec;

struct Wrapper {
    map1: StorageMap<u64, u64>,
    vec2: StorageVec<u64>,
}

storage {
    w: Wrapper = Wrapper { map1: StorageMap {}, vec2: StorageVec {} },
    v: StorageVec<u64> = StorageVec {},
    u: StorageVec<StorageVec<u64>> = StorageVec {},
    map1: StorageMap<u32, u32> = StorageMap{},
    bad_type: StorageVec<Vec<bool>> = StorageVec {},
}

abi MyContract {
    #[storage(read, write)]
    fn main() -> u64;

    #[storage(read)]
    fn return_storage_map() -> StorageMap<u32, u32>;

    #[storage(read)]
    fn return_storage_vec() -> StorageVec<u64>;
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn main() -> u64 {
       let local_map1: StorageMap<u64, u64> = StorageMap {};
       let local_vec1: StorageVec<u64> = StorageVec {};

       local_map1.insert(1, 2);
       local_vec1.push(1);
       storage.w.map1.insert(1, 2);
       storage.w.vec2.push(1);

       let local_map2 = storage.map1;
       local_map2.insert(11, 11);

       1
    }

    #[storage(read)]
    fn return_storage_map() -> StorageMap<u32, u32> {
        storage.map1
    }

    #[storage(read)]
    fn return_storage_vec() -> StorageVec<u64> {
        storage.v
    }
}

#[storage(write)]
fn insert(mapping: StorageMap<u64, u64>) {
    mapping.insert(1, 1);
}

#[storage(read)]
fn return_storage_vec_standalone_fn() -> StorageVec<u64> {
    storage.v
}

pub struct MyStruct { }

impl MyStruct {
    #[storage(read, write)]
    pub fn takes_storage_struct_in_impl(self, my_struct: StorageVec<u64>) {
        my_struct.push(5);
    }
}

pub trait MyTrait {
    #[storage(read, write)]
    fn takes_storage_struct_in_trait_impl(self, my_struct: StorageVec<u64>);
}

impl MyTrait for MyStruct {
    #[storage(read, write)]
    fn takes_storage_struct_in_trait_impl(self, my_struct: StorageVec<u64>) {
        my_struct.push(5);
    }
}
