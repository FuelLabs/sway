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
fn assert_load_vec_impl<T>(
    slot_id_preimage: u64,
)
where
    T: Eq + TestInstance + AbiEncode,
{
    assert_non_zero_sized_type::<T>();

    let vec: StorageKey<StorageVec<T>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));

    let expected_values = T::instances(NUM_OF_ELEMENTS);

    // Load from empty storage vector.
    let mut loaded_vec = vec.load_vec();
    assert(0 == loaded_vec.len());

    // Load full vector content and verify order.
    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        vec.push(expected_values.get(i).unwrap());
        i += 1;
    }

    loaded_vec = vec.load_vec();
    assert(NUM_OF_ELEMENTS == loaded_vec.len());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        assert(expected_values.get(i) == loaded_vec.get(i));
        i += 1;
    }

    // Load after replacing content with a shorter vector.
    let mut shorter_heap_vec = Vec::<T>::new();
    let mut i = 0;
    while i < NUM_OF_ELEMENTS / 2 {
        shorter_heap_vec.push(expected_values.get(i).unwrap());
        i += 1;
    }

    vec.store_vec(shorter_heap_vec);
    loaded_vec = vec.load_vec();
    assert(NUM_OF_ELEMENTS / 2 == loaded_vec.len());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS / 2 {
        assert(expected_values.get(i) == loaded_vec.get(i));
        i += 1;
    }

    // Load single-element vector.
    let _ = vec.clear();
    vec.push(expected_values.get(0).unwrap());

    loaded_vec = vec.load_vec();
    assert(1 == loaded_vec.len());
    assert(expected_values.get(0) == loaded_vec.get(0));
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read, write)]
    fn assert_load_vec() {
        assert_load_vec_impl::<bool>(1);
        assert_load_vec_impl::<u8>(2);
        assert_load_vec_impl::<u16>(3);
        assert_load_vec_impl::<u32>(4);
        assert_load_vec_impl::<u64>(5);
        assert_load_vec_impl::<u256>(6);
        assert_load_vec_impl::<b256>(7);
        assert_load_vec_impl::<raw_slice>(8);
        assert_load_vec_impl::<str>(9);
        assert_load_vec_impl::<str[2]>(10);
        assert_load_vec_impl::<str[5]>(11);
        assert_load_vec_impl::<str[6]>(12);
        assert_load_vec_impl::<str[8]>(13);
        assert_load_vec_impl::<str[12]>(14);
        assert_load_vec_impl::<str[13]>(15);
        assert_load_vec_impl::<[u64; 2]>(16);
        assert_load_vec_impl::<ArrayU8Len2>(17);
        assert_load_vec_impl::<ArrayU8Len5>(18);
        assert_load_vec_impl::<ArrayU8Len6>(19);
        assert_load_vec_impl::<ArrayU8Len8>(20);
        assert_load_vec_impl::<ArrayU8Len12>(21);
        assert_load_vec_impl::<ArrayU8Len13>(22);
        assert_load_vec_impl::<ArrayU64Len3>(23);
        assert_load_vec_impl::<ArrayNestedArrayU8Len2Len3>(24);
        assert_load_vec_impl::<ArrayStructBLen2>(25);
        assert_load_vec_impl::<RawPtrNewtype>(26);
        assert_load_vec_impl::<StructA>(27);
        assert_load_vec_impl::<StructB>(28);
        assert_load_vec_impl::<EnumSingleU8>(29);
        assert_load_vec_impl::<EnumSingleU64>(30);
        assert_load_vec_impl::<EnumSingleBool>(31);
        assert_load_vec_impl::<EnumMultiUnits>(32);
        assert_load_vec_impl::<EnumMultiOneByte>(33);
        assert_load_vec_impl::<EnumU8AndU64>(34);
        assert_load_vec_impl::<EnumQuadSlotSize>(35);
        assert_load_vec_impl::<EnumLargerThanQuadSlot>(36);
        assert_load_vec_impl::<(u8, u32)>(37);
    }

    #[storage(read, write)]
    fn assert_load_vec_nested_storage_type() {
        let vec: StorageKey<StorageVec<StorageVec<u64>>> = StorageKey::new(b256::zero(), 0, b256::zero());

        // Ensure non-empty vector so `load_vec` hits nested-type checks for both implementations.
        vec.resize(1, StorageVec {});
        let _ = vec.load_vec(); // Nested storage types cannot be loaded.
    }

}

#[test]
fn load_vec() {
    let caller = abi(StorageVecLoadVecContractTestsAbi, CONTRACT_ID);
    caller.assert_load_vec();
}

#[test(should_revert)]
fn load_vec_nested_storage_type() {
    let caller = abi(StorageVecLoadVecContractTestsAbi, CONTRACT_ID);
    caller.assert_load_vec_nested_storage_type();
}
