contract;

use std::{hash::*, storage::storage_api::read};

struct MyStorageMap<K, V> where K: Hash {}

impl<K, V> StorageKey<MyStorageMap<K, V>> where K: Hash {
    // This version puts the err on the `vec.push` statement because `vec` is
    // annotated with `Vec<V>`.
    #[storage(read)]
    fn to_vec1(self, key: K) -> Vec<V> {
        let k = sha256((key, self.slot()));
        let len = read::<u64>(k, 0).unwrap_or(0);
        let mut i = 0;
        let mut vec: Vec<V> = Vec::new();
        while len > i {
            let k = sha256((key, i, self.slot()));
            let item = read::<K>(k, 0).unwrap();
            vec.push(item); // <-----
            i += 1;
        }
        vec
    }

    // This version puts the err on the implicit return expression because
    // the type of `vec` (`Vec<K>`) is taken from the `vec.push` statement.
    #[storage(read)]
    fn to_vec2(self, key: K) -> Vec<V> {
        let k = sha256((key, self.slot()));
        let len = read::<u64>(k, 0).unwrap_or(0);
        let mut i = 0;
        let mut vec/*: Vec<V>*/ = Vec::new();
        while len > i {
            let k = sha256((key, i, self.slot()));
            let item = read::<K>(k, 0).unwrap();
            vec.push(item); // <-----
            i += 1;
        }
        vec // <-----
    }
}

storage {
    map1: MyStorageMap<u64, bool> = MyStorageMap::<u64, bool> {},
    map2: MyStorageMap<u64, str[4]> = MyStorageMap::<u64, str[4]> {},
}

abi TestAbi {
    #[storage(read)]
    fn test();
}

impl TestAbi for Contract {
    #[storage(read)]
    fn test() {
        let _ = storage.map1.to_vec1(1u64);
        let _ = storage.map2.to_vec2(1u64);
    }
}
