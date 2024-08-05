contract;

use std::{hash::Hash, storage::storage_string::*};

storage {
    return_storage_int: u64 = 0,
    return_storage_string: StorageString = StorageString {},
    return_storage_map: StorageMap<u64, u64> = StorageMap {},

    store_storage_key_string: StorageMap<StorageKey<StorageString>, u64> = StorageMap {},
    store_storage_key_map: StorageMap<StorageKey<StorageMap<u64, u64>>, u64> = StorageMap {},
    store_storage_key_int: StorageMap<StorageKey<u64>, u64> = StorageMap {},
}

// Tests to ensure this compiles and we can return a StorageKey from a function
abi CompileTestReturnStorageKey {
    fn returns_int() -> StorageKey<u64>;
    fn returns_string() -> StorageKey<StorageString>;
    fn returns_map() -> StorageKey<StorageMap<u64, u64>>;
}

impl CompileTestReturnStorageKey for Contract {
    fn returns_int() -> StorageKey<u64> {
        storage.return_storage_int
    }

    fn returns_string() -> StorageKey<StorageString> {
        storage.return_storage_string
    }

    fn returns_map() -> StorageKey<StorageMap<u64, u64>> {
        storage.return_storage_map
    }
}

// Tests to ensure this compiles and we can store a StorageKey in a StorageMap
abi CompileTestStoreStorageKey {
    #[storage(read, write)]
    fn store_storage_key_u64(storage_key: StorageKey<u64>);
    #[storage(read, write)]
    fn store_storage_key_string(storage_key: StorageKey<StorageString>);
    #[storage(read, write)]
    fn store_storage_key_map(storage_key: StorageKey<StorageMap<u64, u64>>);
}

impl CompileTestStoreStorageKey for Contract {
    #[storage(read, write)]
    fn store_storage_key_u64(storage_key: StorageKey<u64>) {
        storage.store_storage_key_int.insert(storage_key, 1);
    }

    #[storage(read, write)]
    fn store_storage_key_string(storage_key: StorageKey<StorageString>) {
        storage.store_storage_key_string.insert(storage_key, 1);
    }

    #[storage(read, write)]
    fn store_storage_key_map(storage_key: StorageKey<StorageMap<u64, u64>>) {
        storage.store_storage_key_map.insert(storage_key, 1);
    }
}

#[test]
fn storage_key_slot() {
    let key_1 = StorageKey::<u64>::new(b256::min(), u64::zero(), b256::zero());
    assert(key_1.slot() == b256::min());

    let key_2 = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000001,
        u64::zero(),
        b256::zero(),
    );
    assert(
        key_2
            .slot() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let key_3 = StorageKey::<u64>::new(
        b256::max(),
        u64::zero(),
        b256::zero(),
    );
    assert(
        key_3
            .slot() == b256::max(),
    );
}

#[test]
fn storage_key_offset() {
    let key_1 = StorageKey::<u64>::new(b256::zero(), u64::min(), b256::zero());
    assert(key_1.offset() == u64::min());

    let key_2 = StorageKey::<u64>::new(
        b256::zero(),
        1,
        b256::zero(),
    );
    assert(
        key_2
            .offset() == 1,
    );

    let key_3 = StorageKey::<u64>::new(
        b256::zero(),
        u64::max(),
        b256::zero(),
    );
    assert(
        key_3
            .offset() == u64::max(),
    );
}

#[test]
fn storage_key_field_id() {
    let key_1 = StorageKey::<u64>::new(b256::zero(), u64::zero(), b256::min());
    assert(key_1.field_id() == b256::min());

    let key_2 = StorageKey::<u64>::new(
        b256::zero(),
        u64::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        key_2
            .field_id() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let key_3 = StorageKey::<u64>::new(
        b256::zero(),
        u64::zero(),
        b256::max(),
    );
    assert(
        key_3
            .field_id() == b256::max(),
    );
}

#[test]
fn storage_key_new() {
    let key = StorageKey::<u64>::new(b256::min(), u64::min(), b256::min());
    assert(key.slot() == b256::min());
    assert(key.offset() == u64::min());
    assert(key.field_id() == b256::min());

    let key = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000001,
        1,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        key
            .slot() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(key.offset() == 1);
    assert(
        key
            .field_id() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let key = StorageKey::<u64>::new(b256::max(), u64::max(), b256::max());
    assert(key.slot() == b256::max());
    assert(key.offset() == u64::max());
    assert(key.field_id() == b256::max());
}

#[test]
fn storage_key_hash() {
    use std::hash::sha256;
    
    let storage_key_1 = StorageKey::<u64>::zero();
    let digest_1 = sha256(storage_key_1);
    assert(digest_1 == 0x834a709ba2534ebe3ee1397fd4f7bd288b2acc1d20a08d6c862dcd99b6f04400);

    let storage_key_2 = StorageKey::<u64>::new(b256::max(), u64::max(), b256::max());
    let digest_2 = sha256(storage_key_2);
    assert(digest_2 == 0x51cfa32fece0135f38198da2529e7c6a0c1f53747984d55705077b7f6920cc76);

    let storage_key_3 = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000001,
        1,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    let digest_3 = sha256(storage_key_3);
    assert(digest_3 == 0xf86350fc74991ae42f23727a15e80909eb4cfb6cde0ea1664ee73026df3e7f7a);

    let storage_key_4 = StorageKey::<u64>::new(
        0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925, 
        139108464133, 
        0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051
    );
    let digest_4 = sha256(storage_key_4);
    assert(digest_4 != 0x4eaddc8cfcdd27223821e3e31ab54b2416dd3b0c1a86afd7e8d6538ca1bd0a77);
}

#[test]
fn storage_key_zero() {
    let storage_key = StorageKey::<u64>::zero();
    assert(
        storage_key
            .slot() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert(
        storage_key
            .offset() == 0,
    );
    assert(
        storage_key
            .field_id() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
}

#[test]
fn storage_key_is_zero() {
    let zero_storage_key = StorageKey::<u64>::zero();
    assert(zero_storage_key.is_zero());

    let storage_key_2 = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert(!storage_key_2.is_zero());

    let storage_key_3 = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000000,
        1,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert(!storage_key_3.is_zero());

    let storage_key_4 = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(!storage_key_4.is_zero());

    let storage_key_5 = StorageKey::<u64>::new(b256::max(), u64::max(), b256::max());
    assert(!storage_key_5.is_zero());
}

#[test()]
fn storage_key_encode_and_decode() {
    let storage_key_1 = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000001,
        1,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    );

    let storage_key_2 = abi_decode::<StorageKey<u64>>(encode(storage_key_1));

    assert(
        storage_key_2
            .slot() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        storage_key_2
            .offset() == 1,
    );
    assert(
        storage_key_2
            .field_id() == 0x0000000000000000000000000000000000000000000000000000000000000002,
    );
}
