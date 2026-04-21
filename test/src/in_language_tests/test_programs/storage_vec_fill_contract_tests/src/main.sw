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
fn assert_fill_impl<T>(
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

    let fill_value = expected_values.get(NUM_OF_ELEMENTS / 2).unwrap();
    vec.fill(fill_value);
    assert(NUM_OF_ELEMENTS == vec.len());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        assert(fill_value == vec.get(i).unwrap().read());
        i += 1;
    }

    // Filling empty and single-element vectors should be a no-op / simple overwrite.
    let _ = vec.clear();
    vec.fill(fill_value);
    assert(0 == vec.len());

    vec.push(expected_values.get(0).unwrap());
    vec.fill(fill_value);
    assert(1 == vec.len());
    assert(fill_value == vec.get(0).unwrap().read());
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read, write)]
    fn assert_fill() {
        assert_fill_impl::<bool>(1);
        assert_fill_impl::<u8>(2);
        assert_fill_impl::<u16>(3);
        assert_fill_impl::<u32>(4);
        assert_fill_impl::<u64>(5);
        assert_fill_impl::<u256>(6);
        assert_fill_impl::<b256>(7);
        assert_fill_impl::<raw_slice>(8);
        assert_fill_impl::<str>(9);
        assert_fill_impl::<str[2]>(10);
        assert_fill_impl::<str[6]>(11);
        assert_fill_impl::<str[8]>(12);
        assert_fill_impl::<str[12]>(13);
        assert_fill_impl::<[u64; 2]>(14);
        assert_fill_impl::<ArrayU8Len2>(15);
        assert_fill_impl::<ArrayU64Len3>(16);
        assert_fill_impl::<ArrayNestedArrayU8Len2Len3>(17);
        assert_fill_impl::<ArrayStructBLen2>(18);
        assert_fill_impl::<RawPtrNewtype>(19);
        assert_fill_impl::<StructA>(20);
        assert_fill_impl::<StructB>(21);
        assert_fill_impl::<EnumSingleU8>(22);
        assert_fill_impl::<EnumSingleU64>(23);
        assert_fill_impl::<EnumSingleBool>(24);
        assert_fill_impl::<EnumMultiUnits>(25);
        assert_fill_impl::<EnumMultiOneByte>(26);
        assert_fill_impl::<EnumU8AndU64>(27);
        assert_fill_impl::<EnumQuadSlotSize>(28);
        assert_fill_impl::<EnumLargerThanQuadSlot>(29);
        assert_fill_impl::<(u8, u32)>(30);
    }

    #[storage(write)]
    fn assert_fill_nested_storage_type() {
        let vec: StorageKey<StorageVec<StorageVec<u64>>> = StorageKey::new(b256::zero(), 0, b256::zero());
        vec.fill(StorageVec {}); // Nested storage types cannot be filled.
    }

}

#[test]
fn fill() {
    let caller = abi(StorageVecFillContractTestsAbi, CONTRACT_ID);
    caller.assert_fill();
}

#[test(should_revert)]
fn fill_nested_storage_type() {
    let caller = abi(StorageVecFillContractTestsAbi, CONTRACT_ID);
    caller.assert_fill_nested_storage_type();
}
