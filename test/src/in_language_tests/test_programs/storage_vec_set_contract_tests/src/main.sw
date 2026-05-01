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
fn assert_set_impl<T>(
    slot_id_preimage: u64,
)
where
    T: Eq + TestInstance + AbiEncode,
{
    assert_non_zero_sized_type::<T>();

    let vec: StorageKey<StorageVec<T>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        vec.push(T::default());
        i += 1;
    }

    let expected_values = T::instances(NUM_OF_ELEMENTS);

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        vec.set(i, expected_values.get(i).unwrap());
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        assert(NUM_OF_ELEMENTS == vec.len());
        i += 1;
    }

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
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
    fn assert_set() {
        assert_set_impl::<bool>(1);
        assert_set_impl::<u8>(2);
        assert_set_impl::<u16>(3);
        assert_set_impl::<u32>(4);
        assert_set_impl::<u64>(5);
        assert_set_impl::<u256>(6);
        assert_set_impl::<b256>(7);
        assert_set_impl::<raw_slice>(8);
        assert_set_impl::<str>(9);
        assert_set_impl::<str[2]>(10);
        assert_set_impl::<str[5]>(11);
        assert_set_impl::<str[6]>(12);
        assert_set_impl::<str[8]>(13);
        assert_set_impl::<str[12]>(14);
        assert_set_impl::<str[13]>(15);
        assert_set_impl::<[u64; 2]>(16);
        assert_set_impl::<ArrayU8Len2>(17);
        assert_set_impl::<ArrayU8Len5>(18);
        assert_set_impl::<ArrayU8Len6>(19);
        assert_set_impl::<ArrayU8Len8>(20);
        assert_set_impl::<ArrayU8Len12>(21);
        assert_set_impl::<ArrayU8Len13>(22);
        assert_set_impl::<ArrayU64Len3>(23);
        assert_set_impl::<ArrayNestedArrayU8Len2Len3>(24);
        assert_set_impl::<ArrayStructBLen2>(25);
        assert_set_impl::<RawPtrNewtype>(26);
        assert_set_impl::<StructA>(27);
        assert_set_impl::<StructB>(28);
        assert_set_impl::<EnumSingleU8>(29);
        assert_set_impl::<EnumSingleU64>(30);
        assert_set_impl::<EnumSingleBool>(31);
        assert_set_impl::<EnumMultiUnits>(32);
        assert_set_impl::<EnumMultiOneByte>(33);
        assert_set_impl::<EnumU8AndU64>(34);
        assert_set_impl::<EnumQuadSlotSize>(35);
        assert_set_impl::<EnumLargerThanQuadSlot>(36);
        assert_set_impl::<(u8, u32)>(37);
    }

    #[storage(read, write)]
    fn assert_set_index_out_of_bounds() {
        let vec: StorageKey<StorageVec<u64>> = StorageKey::new(b256::zero(), 0, b256::zero());
        vec.set(0, 42);
    }

    #[storage(read, write)]
    fn assert_set_nested_storage_type() {
        let vec: StorageKey<StorageVec<StorageVec<u64>>> = StorageKey::new(b256::zero(), 0, b256::zero());
        vec.push(StorageVec {});
        vec.set(0, StorageVec {}); // Nested storage types cannot be set.
    }

}

#[test]
fn set() {
    let caller = abi(StorageVecSetContractTestsAbi, CONTRACT_ID);
    caller.assert_set();
}

#[test(should_revert)]
fn set_out_of_bounds() {
    let caller = abi(StorageVecSetContractTestsAbi, CONTRACT_ID);
    caller.assert_set_index_out_of_bounds();
}

#[test(should_revert)]
fn set_nested_storage_type() {
    let caller = abi(StorageVecSetContractTestsAbi, CONTRACT_ID);
    caller.assert_set_nested_storage_type();
}
