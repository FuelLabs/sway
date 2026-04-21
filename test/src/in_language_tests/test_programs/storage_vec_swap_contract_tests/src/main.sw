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
fn assert_swap_impl<T>(
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

    // Swap the first and last elements.
    vec.swap(0, NUM_OF_ELEMENTS - 1);
    assert(expected_values.get(NUM_OF_ELEMENTS - 1) == vec.get(0).unwrap().try_read());
    assert(expected_values.get(0) == vec.get(NUM_OF_ELEMENTS - 1).unwrap().try_read());
    assert(NUM_OF_ELEMENTS == vec.len());

    let mut i = 1;
    while i < NUM_OF_ELEMENTS - 1 {
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    // Swap two middle elements and verify only those positions change.
    let middle_left = NUM_OF_ELEMENTS / 2 - 1;
    let middle_right = NUM_OF_ELEMENTS / 2 + 1;
    vec.swap(middle_left, middle_right);

    assert(expected_values.get(middle_right) == vec.get(middle_left).unwrap().try_read());
    assert(expected_values.get(middle_left) == vec.get(middle_right).unwrap().try_read());
    assert(expected_values.get(NUM_OF_ELEMENTS - 1) == vec.get(0).unwrap().try_read());
    assert(expected_values.get(0) == vec.get(NUM_OF_ELEMENTS - 1).unwrap().try_read());
    assert(NUM_OF_ELEMENTS == vec.len());

    let mut i = 1;
    while i < NUM_OF_ELEMENTS - 1 {
        if i != middle_left && i != middle_right {
            assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        }
        i += 1;
    }

    // Swapping an element with itself is a no-op.
    let unchanged_index = NUM_OF_ELEMENTS / 2;
    let before = vec.get(unchanged_index).unwrap().try_read();
    vec.swap(unchanged_index, unchanged_index);
    assert(before == vec.get(unchanged_index).unwrap().try_read());
    assert(NUM_OF_ELEMENTS == vec.len());
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read, write)]
    fn assert_swap() {
        assert_swap_impl::<bool>(1);
        assert_swap_impl::<u8>(2);
        assert_swap_impl::<u16>(3);
        assert_swap_impl::<u32>(4);
        assert_swap_impl::<u64>(5);
        assert_swap_impl::<u256>(6);
        assert_swap_impl::<b256>(7);
        assert_swap_impl::<raw_slice>(8);
        assert_swap_impl::<str>(9);
        assert_swap_impl::<str[2]>(10);
        assert_swap_impl::<str[6]>(11);
        assert_swap_impl::<str[8]>(12);
        assert_swap_impl::<str[12]>(13);
        assert_swap_impl::<[u64; 2]>(14);
        assert_swap_impl::<ArrayU8Len2>(15);
        assert_swap_impl::<ArrayU64Len3>(16);
        assert_swap_impl::<ArrayNestedArrayU8Len2Len3>(17);
        assert_swap_impl::<ArrayStructBLen2>(18);
        assert_swap_impl::<RawPtrNewtype>(19);
        assert_swap_impl::<StructA>(20);
        assert_swap_impl::<StructB>(21);
        assert_swap_impl::<EnumSingleU8>(22);
        assert_swap_impl::<EnumSingleU64>(23);
        assert_swap_impl::<EnumSingleBool>(24);
        assert_swap_impl::<EnumMultiUnits>(25);
        assert_swap_impl::<EnumMultiOneByte>(26);
        assert_swap_impl::<EnumU8AndU64>(27);
        assert_swap_impl::<EnumQuadSlotSize>(28);
        assert_swap_impl::<EnumLargerThanQuadSlot>(29);
        assert_swap_impl::<(u8, u32)>(30);
    }

    #[storage(write)]
    fn assert_swap_index_out_of_bounds() {
        let vec: StorageKey<StorageVec<u64>> = StorageKey::new(b256::zero(), 0, b256::zero());
        vec.swap(0, 1);
    }

    #[storage(write)]
    fn assert_swap_nested_storage_type() {
        let vec: StorageKey<StorageVec<StorageVec<u64>>> = StorageKey::new(b256::zero(), 0, b256::zero());
        vec.push(StorageVec {});
        vec.push(StorageVec {});
        vec.swap(0, 1); // Nested storage types cannot be swapped.
    }

}

#[test]
fn swap() {
    let caller = abi(StorageVecSwapContractTestsAbi, CONTRACT_ID);
    caller.assert_swap();
}

#[test(should_revert)]
fn swap_out_of_bounds() {
    let caller = abi(StorageVecSwapContractTestsAbi, CONTRACT_ID);
    caller.assert_swap_index_out_of_bounds();
}

#[test(should_revert)]
fn swap_nested_storage_type() {
    let caller = abi(StorageVecSwapContractTestsAbi, CONTRACT_ID);
    caller.assert_swap_nested_storage_type();
}
