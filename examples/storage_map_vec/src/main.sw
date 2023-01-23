contract;

use storagemapvec::StorageMapVec;
storage {
    mapvec: StorageMapVec<u64, u64> = StorageMapVec {},
}
abi MyContract {
    #[storage(read, write)]
    fn push(key: u64, value: u64);
    #[storage(read)]
    fn get(key: u64, index: u64);
}
impl MyContract for Contract {
    #[storage(read, write)]
    fn push(key: u64, value: u64) {
        // this will push the value to the vec, which is accessible with the key
        storage.mapvec.push(key, value);
    }
    #[storage(read)]
    fn get(key: u64, index: u64) {
        // this will retrieve the vec at given key, and then retrieve the value at given index from that vec
        storage.mapvec.get(key, index)
    }
}