contract;

use std::storage::{StorageMap, StorageVec};

struct Wrapper {
    map1: StorageMap<u64, u64>,
    vec2: StorageVec<u64>,
}

storage {
    w: Wrapper = Wrapper { map1: StorageMap {}, vec2: StorageVec {} },
    v: StorageVec<u64> = StorageVec {},
    u: StorageVec<StorageVec<u64>> = StorageVec {},
    map1: StorageMap<u32, u32> = StorageMap{},
}

impl Contract {
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
}

#[storage(write)]
fn insert(mapping: StorageMap<u64, u64>) {
    mapping.insert(1, 1);
}
