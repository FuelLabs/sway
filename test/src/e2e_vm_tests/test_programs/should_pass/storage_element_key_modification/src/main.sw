contract;

use std::hash::*;

storage {
    a: u8 = 0,
}

impl Contract {
    #[storage(read)]
    fn storage_key_address() {
        let storage_key_address = __addr_of(storage.a);
        assert(!storage_key_address.is_null());
        let storage_key = storage_key_address.read::<StorageKey<()>>();
        assert_eq(storage_key.slot(), get_storage_field_slot("storage.a"));
        assert_eq(storage_key.slot(), storage_key.field_id());
        assert_eq(storage_key.offset(), 0);
    }

    #[storage(read)]
    fn storage_key_modification() {
        let storage_key_address = __addr_of(storage.a);
        // Attempting to modify the storage key address must cause a revert.
        // It's an attempt to modify a constant value in the data section.
        storage_key_address.write(42u8);
    }
}

/// Computes the storage slot for a given field path using the same hashing
/// algorithm as the compiler.
/// The compiler hashes only the field path content, without its length as a prefix.
fn get_storage_field_slot(field_path: str) -> b256 {
    let mut hasher = Hasher::new();
    hasher.write_u8(0u8); // Domain discriminator for storage field keys.
    hasher.write_str(field_path);
    hasher.sha256()
}

#[test]
fn test_storage_key_address() {
    let caller = abi(StorageElementKeyModificationAbi, CONTRACT_ID);
    caller.storage_key_address();
}

#[test(should_revert)]
fn test_storage_key_modification() {
    let caller = abi(StorageElementKeyModificationAbi, CONTRACT_ID);
    caller.storage_key_modification();
}