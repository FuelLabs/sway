contract;
use basic_storage_abi::*;
use core::storable::*;

struct S {
    a: u64,
//    b: u64, // once we have addition for b256
}

impl core::storable::Storable for S {
    fn write(self, key: b256) {
        self.a.write(key);
        // self.b.write(key + 1); // once we have addition for b256
    }
    fn read(key: b256) -> S {
        S {
            a: ~u64::read(key),
            // b: ~u64::read(key + 1), // once we have addition for b256
        }
    }
}

storage {
    x: u64,
    y: u64,
    s: S,
}

impl StoreU64 for Contract {
    fn get_u64(storage_key: b256) -> u64 {
        let sum1 = storage.x + storage.y;
        let local_s = storage.s; 
        local_s.a + sum1
    }

    fn store_u64(key: b256, value: u64) {
        storage.x = value;
        storage.y = value;
        storage.s = S { a: value };
    }
}
