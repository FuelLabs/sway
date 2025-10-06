contract;

use std::hash::*;

storage {
    a: u8 = 0,
    b: u8 = 0,
    ns1 {
        a: u8 = 0,
        b: u8 = 0,
    },
    ns2 {
        ns3 {
            a: u8 = 0,
            b: u8 = 0,
        },
    },
}

abi TestStorageKeyCalculation {
    #[storage(read)]
    fn test_storage_key_calculation();
}

impl TestStorageKeyCalculation for Contract {
    #[storage(read)]
    fn test_storage_key_calculation() {
        assert_eq(storage.a.slot(), get_storage_field_slot("storage.a"));
        assert_eq(storage.b.slot(), get_storage_field_slot("storage.b"));
        assert_eq(storage::ns1.a.slot(), get_storage_field_slot("storage::ns1.a"));
        assert_eq(storage::ns1.b.slot(), get_storage_field_slot("storage::ns1.b"));
        assert_eq(storage::ns2::ns3.a.slot(), get_storage_field_slot("storage::ns2::ns3.a"));
        assert_eq(storage::ns2::ns3.b.slot(), get_storage_field_slot("storage::ns2::ns3.b"));
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
fn test() {
    let caller = abi(TestStorageKeyCalculation, CONTRACT_ID);
    caller.test_storage_key_calculation();
}