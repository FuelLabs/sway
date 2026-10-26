contract;

use std::storage::storage_vec::*;

struct RefBox {
    value: &u64,
}

storage {
    direct: &u64 = &0,
    nested: RefBox = RefBox { value: &0 },
    map: StorageMap<u64, RefBox> = StorageMap {},
    vec: StorageVec<RefBox> = StorageVec {},
}
