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
fn assert_remove_impl<T>(
    slot_id_preimage: u64,
)
where
    T: Eq + TestInstance + AbiEncode,
{
    assert_non_zero_sized_type::<T>();

    let vec: StorageKey<StorageVec<T>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));

    let expected_values = T::instances(NUM_OF_ELEMENTS);

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        vec.push(expected_values.get(i).unwrap());
        i += 1;
    }

    // Remove front element and verify left shift.
    let removed_front = vec.remove(0);
    assert(expected_values.get(0).unwrap() == removed_front);
    assert(NUM_OF_ELEMENTS - 1 == vec.len());

    let mut i = 0;
    while i < vec.len() {
        assert(expected_values.get(i + 1) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    // Remove middle element and verify left shift around removed index.
    let middle_index = vec.len() / 2;
    let removed_middle = vec.remove(middle_index);
    assert(expected_values.get(middle_index + 1).unwrap() == removed_middle);
    assert(NUM_OF_ELEMENTS - 2 == vec.len());

    let mut i = 0;
    while i < middle_index {
        assert(expected_values.get(i + 1) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    let mut i = middle_index;
    while i < vec.len() {
        assert(expected_values.get(i + 2) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    // Remove last element.
    let removed_last = vec.remove(vec.len() - 1);
    assert(expected_values.get(NUM_OF_ELEMENTS - 1).unwrap() == removed_last);
    assert(NUM_OF_ELEMENTS - 3 == vec.len());

    // Verify final content after front, middle and last removals.
    let mut i = 0;
    while i < middle_index {
        assert(expected_values.get(i + 1) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    let mut i = middle_index;
    while i < vec.len() {
        assert(expected_values.get(i + 2) == vec.get(i).unwrap().try_read());
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
    fn assert_remove() {
        assert_remove_impl::<bool>(1);
        assert_remove_impl::<u8>(2);
        assert_remove_impl::<u16>(3);
        assert_remove_impl::<u32>(4);
        assert_remove_impl::<u64>(5);
        assert_remove_impl::<u256>(6);
        assert_remove_impl::<b256>(7);
        assert_remove_impl::<raw_slice>(8);
        assert_remove_impl::<str>(9);
        assert_remove_impl::<str[2]>(10);
        assert_remove_impl::<str[5]>(11);
        assert_remove_impl::<str[6]>(12);
        assert_remove_impl::<str[8]>(13);
        assert_remove_impl::<str[12]>(14);
        assert_remove_impl::<str[13]>(15);
        assert_remove_impl::<[u64; 2]>(16);
        assert_remove_impl::<ArrayU8Len2>(17);
        assert_remove_impl::<ArrayU8Len5>(18);
        assert_remove_impl::<ArrayU8Len6>(19);
        assert_remove_impl::<ArrayU8Len8>(20);
        assert_remove_impl::<ArrayU8Len12>(21);
        assert_remove_impl::<ArrayU8Len13>(22);
        assert_remove_impl::<ArrayU64Len3>(23);
        assert_remove_impl::<ArrayNestedArrayU8Len2Len3>(24);
        assert_remove_impl::<ArrayStructBLen2>(25);
        assert_remove_impl::<RawPtrNewtype>(26);
        assert_remove_impl::<StructA>(27);
        assert_remove_impl::<StructB>(28);
        assert_remove_impl::<EnumSingleU8>(29);
        assert_remove_impl::<EnumSingleU64>(30);
        assert_remove_impl::<EnumSingleBool>(31);
        assert_remove_impl::<EnumMultiUnits>(32);
        assert_remove_impl::<EnumMultiOneByte>(33);
        assert_remove_impl::<EnumU8AndU64>(34);
        assert_remove_impl::<EnumQuadSlotSize>(35);
        assert_remove_impl::<EnumLargerThanQuadSlot>(36);
        assert_remove_impl::<(u8, u32)>(37);
    }

    #[storage(read, write)]
    fn assert_remove_index_out_of_bounds() {
        let vec: StorageKey<StorageVec<u64>> = StorageKey::new(b256::zero(), 0, b256::zero());
        let _ = vec.remove(0);
    }

    #[storage(read, write)]
    fn assert_remove_nested_storage_type() {
        let vec: StorageKey<StorageVec<StorageVec<u64>>> = StorageKey::new(b256::zero(), 0, b256::zero());
        vec.push(StorageVec {});
        let _ = vec.remove(0); // Nested storage types cannot be removed.
    }

}

#[test]
fn remove() {
    let caller = abi(StorageVecRemoveContractTestsAbi, CONTRACT_ID);
    caller.assert_remove();
}

#[test(should_revert)]
fn remove_out_of_bounds() {
    let caller = abi(StorageVecRemoveContractTestsAbi, CONTRACT_ID);
    caller.assert_remove_index_out_of_bounds();
}

#[test(should_revert)]
fn remove_nested_storage_type() {
    let caller = abi(StorageVecRemoveContractTestsAbi, CONTRACT_ID);
    caller.assert_remove_nested_storage_type();
}
