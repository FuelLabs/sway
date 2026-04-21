// TODO: The initial reason for having several `storage_vec` tests is
//       reaching the limit in the data section size of a single contract
//       that contained all the tests for `StorageVec<T>` methods.
//       See: https://github.com/FuelLabs/sway/issues/7612
//       But even when the issue is solved, it perhaps still makes sense to have
//       separate test projects for better organization and readability
//       of the tests.
// TODO: Replace `assert(x == y)` back with `assert_eq(x, y)` once `assert_eq` no longer
//       causes data-section explosion. See also: https://github.com/FuelLabs/sway/issues/7612
contract;

use test_types::*;

use std::hash::{Hash, sha256};
use std::storage::storage_vec::*;

/// Default number of vector elements to use in tests.
#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
const NUM_OF_ELEMENTS: u64 = 11;

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[storage(read, write)]
fn assert_insert_impl<T>(
    slot_id_preimage: u64,
)
where
    T: Eq + TestInstance + AbiEncode,
{
    assert_non_zero_sized_type::<T>();

    let vec: StorageKey<StorageVec<T>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));

    let expected_values = T::instances(NUM_OF_ELEMENTS);

    // Append instances.
    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        vec.insert(i, expected_values.get(i).unwrap());
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        assert(i + 1 == vec.len());
        i += 1;
    }

    assert(NUM_OF_ELEMENTS == vec.len());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    // Insert front element and verify right shift.
    let inserted_front = T::default();
    vec.insert(0, inserted_front);
    assert(NUM_OF_ELEMENTS + 1 == vec.len());
    assert(inserted_front == vec.get(0).unwrap().read());
    assert(expected_values.get(0) == vec.get(1).unwrap().try_read());

    // Insert middle element and verify right shift around inserted index.
    let inserted_middle = expected_values.get(NUM_OF_ELEMENTS / 2).unwrap();
    let middle_index = vec.len() / 2;
    let old_middle = vec.get(middle_index).unwrap().try_read();
    vec.insert(middle_index, inserted_middle);
    assert(NUM_OF_ELEMENTS + 2 == vec.len());
    assert(inserted_middle == vec.get(middle_index).unwrap().read());
    assert(old_middle == vec.get(middle_index + 1).unwrap().try_read());

    // Insert last element and verify it is appended correctly.
    let inserted_end = expected_values.get(NUM_OF_ELEMENTS - 1).unwrap();
    vec.insert(vec.len(), inserted_end);
    assert(NUM_OF_ELEMENTS + 3 == vec.len());
    assert(inserted_end == vec.last().unwrap().read());

    // Verify full expected sequence after all insertions.
    let mut expected_after_insertions = Vec::new();
    expected_after_insertions.push(inserted_front);

    let mut i = 0;
    while i < middle_index {
        expected_after_insertions.push(expected_values.get(i).unwrap());
        i += 1;
    }

    expected_after_insertions.push(inserted_middle);

    let mut i = middle_index;
    while i < NUM_OF_ELEMENTS {
        expected_after_insertions.push(expected_values.get(i).unwrap());
        i += 1;
    }

    expected_after_insertions.push(inserted_end);

    let mut i = 0;
    while i < expected_after_insertions.len() {
        assert(expected_after_insertions.get(i) == vec.get(i).unwrap().try_read());
        i += 1;
    }
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read, write)]
    fn assert_insert() {
        assert_insert_impl::<bool>(1);
        assert_insert_impl::<u8>(2);
        assert_insert_impl::<u16>(3);
        assert_insert_impl::<u32>(4);
        assert_insert_impl::<u64>(5);
        assert_insert_impl::<u256>(6);
        assert_insert_impl::<b256>(7);
        assert_insert_impl::<raw_slice>(8);
        assert_insert_impl::<str>(9);
        assert_insert_impl::<str[2]>(10);
        assert_insert_impl::<str[6]>(11);
        assert_insert_impl::<str[8]>(12);
        assert_insert_impl::<str[12]>(13);
        assert_insert_impl::<[u64; 2]>(14);
        assert_insert_impl::<ArrayU8Len2>(15);
        assert_insert_impl::<ArrayU64Len3>(16);
        assert_insert_impl::<ArrayNestedArrayU8Len2Len3>(17);
        assert_insert_impl::<ArrayStructBLen2>(18);
        assert_insert_impl::<RawPtrNewtype>(19);
        assert_insert_impl::<StructA>(20);
        assert_insert_impl::<StructB>(21);
        assert_insert_impl::<EnumSingleU8>(22);
        assert_insert_impl::<EnumSingleU64>(23);
        assert_insert_impl::<EnumSingleBool>(24);
        assert_insert_impl::<EnumMultiUnits>(25);
        assert_insert_impl::<EnumMultiOneByte>(26);
        assert_insert_impl::<EnumU8AndU64>(27);
        assert_insert_impl::<EnumQuadSlotSize>(28);
        assert_insert_impl::<EnumLargerThanQuadSlot>(29);
        assert_insert_impl::<(u8, u32)>(30);
    }

    #[storage(read, write)]
    fn assert_insert_index_out_of_bounds() {
        let vec: StorageKey<StorageVec<u64>> = StorageKey::new(b256::zero(), 0, b256::zero());
        vec.insert(1, 42);
    }

    #[storage(write)]
    fn assert_insert_nested_storage_type() {
        let vec: StorageKey<StorageVec<StorageVec<u64>>> = StorageKey::new(b256::zero(), 0, b256::zero());
        vec.insert(0, StorageVec {}); // Nested storage types cannot be inserted.
    }

}

#[test]
fn insert() {
    let caller = abi(StorageVecInsertContractTestsAbi, CONTRACT_ID);
    caller.assert_insert();
}

#[test(should_revert)]
fn insert_out_of_bounds() {
    let caller = abi(StorageVecInsertContractTestsAbi, CONTRACT_ID);
    caller.assert_insert_index_out_of_bounds();
}

#[test(should_revert)]
fn insert_nested_storage_type() {
    let caller = abi(StorageVecInsertContractTestsAbi, CONTRACT_ID);
    caller.assert_insert_nested_storage_type();
}
