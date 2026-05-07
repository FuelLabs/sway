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
fn assert_swap_remove_impl<T>(
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

    // Swap-remove front element: it should be replaced by the last, and length decremented.
    let removed_front = vec.swap_remove(0);
    assert(expected_values.get(0).unwrap() == removed_front);
    assert(NUM_OF_ELEMENTS - 1 == vec.len());
    // The last element (index NUM_OF_ELEMENTS - 1) should now be at index 0.
    assert(expected_values.get(NUM_OF_ELEMENTS - 1) == vec.get(0).unwrap().try_read());
    // All other elements (1..NUM_OF_ELEMENTS-1) remain at their original positions.
    let mut i = 1;
    while i < vec.len() {
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    // Swap-remove middle element.
    let middle_index = vec.len() / 2;
    let removed_middle = vec.swap_remove(middle_index);
    assert(expected_values.get(middle_index).unwrap() == removed_middle);
    assert(NUM_OF_ELEMENTS - 2 == vec.len());
    // The element that was last before this removal (index NUM_OF_ELEMENTS - 2 of the prior state,
    // which is expected_values[NUM_OF_ELEMENTS - 2]) is now at middle_index.
    assert(expected_values.get(NUM_OF_ELEMENTS - 2) == vec.get(middle_index).unwrap().try_read());

    // Swap-remove last element: it should simply be removed without any swap.
    let last_index = vec.len() - 1;
    let removed_last = vec.swap_remove(last_index);
    assert(expected_values.get(last_index).unwrap() == removed_last);
    assert(NUM_OF_ELEMENTS - 3 == vec.len());

    // Swap-remove when there's only one element left: it should be removed and vector becomes empty.
    let _ = vec.clear();
    vec.push(expected_values.get(0).unwrap());
    let removed_only = vec.swap_remove(0);
    assert(expected_values.get(0).unwrap() == removed_only);
    assert(0 == vec.len());
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read, write)]
    fn assert_swap_remove() {
        assert_swap_remove_impl::<bool>(1);
        assert_swap_remove_impl::<u8>(2);
        assert_swap_remove_impl::<u16>(3);
        assert_swap_remove_impl::<u32>(4);
        assert_swap_remove_impl::<u64>(5);
        assert_swap_remove_impl::<u256>(6);
        assert_swap_remove_impl::<b256>(7);
        assert_swap_remove_impl::<raw_slice>(8);
        assert_swap_remove_impl::<str>(9);
        assert_swap_remove_impl::<str[2]>(10);
        assert_swap_remove_impl::<str[5]>(11);
        assert_swap_remove_impl::<str[6]>(12);
        assert_swap_remove_impl::<str[8]>(13);
        assert_swap_remove_impl::<str[12]>(14);
        assert_swap_remove_impl::<str[13]>(15);
        assert_swap_remove_impl::<[u64; 2]>(16);
        assert_swap_remove_impl::<ArrayU8Len2>(17);
        assert_swap_remove_impl::<ArrayU8Len5>(18);
        assert_swap_remove_impl::<ArrayU8Len6>(19);
        assert_swap_remove_impl::<ArrayU8Len8>(20);
        assert_swap_remove_impl::<ArrayU8Len12>(21);
        assert_swap_remove_impl::<ArrayU8Len13>(22);
        assert_swap_remove_impl::<ArrayU64Len3>(23);
        assert_swap_remove_impl::<ArrayNestedArrayU8Len2Len3>(24);
        assert_swap_remove_impl::<ArrayStructBLen2>(25);
        assert_swap_remove_impl::<RawPtrNewtype>(26);
        assert_swap_remove_impl::<StructA>(27);
        assert_swap_remove_impl::<StructB>(28);
        assert_swap_remove_impl::<EnumSingleU8>(29);
        assert_swap_remove_impl::<EnumSingleU64>(30);
        assert_swap_remove_impl::<EnumSingleBool>(31);
        assert_swap_remove_impl::<EnumMultiUnits>(32);
        assert_swap_remove_impl::<EnumMultiOneByte>(33);
        assert_swap_remove_impl::<EnumU8AndU64>(34);
        assert_swap_remove_impl::<EnumQuadSlotSize>(35);
        assert_swap_remove_impl::<EnumLargerThanQuadSlot>(36);
        assert_swap_remove_impl::<(u8, u32)>(37);
    }

    #[storage(read, write)]
    fn assert_swap_remove_index_out_of_bounds() {
        let vec: StorageKey<StorageVec<u64>> = StorageKey::new(b256::zero(), 0, b256::zero());
        let _ = vec.swap_remove(0);
    }

    #[storage(write)]
    fn assert_swap_remove_nested_storage_type() {
        let vec: StorageKey<StorageVec<StorageVec<u64>>> = StorageKey::new(b256::zero(), 0, b256::zero());
        vec.push(StorageVec {});
        let _ = vec.swap_remove(0); // Nested storage types cannot be swap-removed.
    }

}

#[test]
fn swap_remove() {
    let caller = abi(StorageVecSwapRemoveContractTestsAbi, CONTRACT_ID);
    caller.assert_swap_remove();
}

#[test(should_revert)]
fn swap_remove_out_of_bounds() {
    let caller = abi(StorageVecSwapRemoveContractTestsAbi, CONTRACT_ID);
    caller.assert_swap_remove_index_out_of_bounds();
}

#[test(should_revert)]
fn swap_remove_nested_storage_type() {
    let caller = abi(StorageVecSwapRemoveContractTestsAbi, CONTRACT_ID);
    caller.assert_swap_remove_nested_storage_type();
}
