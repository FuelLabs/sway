contract;

use std::{
    hash::sha256,
    storage::get
};

struct MyStorageMap<K, V> { }

impl<K, V> MyStorageMap<K, V> {
    // This version puts the err on the `vec.push` statement because `vec` is
    // annotated with `Vec<V>`.

    #[storage(read)]
    fn to_vec1(self, key: K) -> Vec<V> {
        let k = sha256((key, __get_storage_key()));
        let len = get::<u64>(k);
        let mut i = 0;
        let mut vec: Vec<V> = Vec::new();
        while len > i {
            let k = sha256((key, i, __get_storage_key()));
            let item = get::<K>(k);
            vec.push(item); // <-----
            i += 1;
        }
        vec
    }

    // This version puts the err on the implicit return expression because
    // the type of `vec` (`Vec<K>`) is taken from the `vec.push` statement.

    #[storage(read)]
    fn to_vec2(self, key: K) -> Vec<V> {
        let k = sha256((key, __get_storage_key()));
        let len = get::<u64>(k);
        let mut i = 0;
        let mut vec/*: Vec<V>*/ = Vec::new();
        while len > i {
            let k = sha256((key, i, __get_storage_key()));
            let item = get::<K>(k);
            vec.push(item); // <-----
            i += 1;
        }
        vec // <-----
    }
}

storage {
    map1: MyStorageMap<u64, bool> = MyStorageMap {},
    map2: MyStorageMap<u64, str[4]> = MyStorageMap {},
}

abi TestAbi {
    #[storage(read)]
    fn test();
}

impl TestAbi for Contract {
    #[storage(read)]
    fn test() {
        let y = storage.map1.to_vec1(1u64);
        let z = storage.map2.to_vec2(1u64);
    }
}
