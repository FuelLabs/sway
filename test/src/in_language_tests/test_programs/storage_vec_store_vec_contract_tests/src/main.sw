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
#[storage(write)]
fn assert_store_vec_impl<T>(
    slot_id_preimage: u64,
)
where
    T: Eq + TestInstance + AbiEncode,
{
    assert_non_zero_sized_type::<T>();

    let vec: StorageKey<StorageVec<T>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));

    let expected_values = T::instances(NUM_OF_ELEMENTS);

    // Store vector with content into empty storage vector and verify.
    let mut heap_vec = Vec::<T>::new();
    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        heap_vec.push(expected_values.get(i).unwrap());
        i += 1;
    }

    vec.store_vec(heap_vec);
    assert(NUM_OF_ELEMENTS == vec.len());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    // Store vector with less content into existing storage vector and verify.
    heap_vec.resize(NUM_OF_ELEMENTS / 2, T::default());
    vec.store_vec(heap_vec);
    assert(NUM_OF_ELEMENTS / 2 == vec.len());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS / 2 {
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    // Store vector with more content into existing storage vector and verify.
    heap_vec.resize(NUM_OF_ELEMENTS, T::default());
    vec.store_vec(heap_vec);
    assert(NUM_OF_ELEMENTS == vec.len());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS / 2 {
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    let mut i = NUM_OF_ELEMENTS / 2;
    while i < NUM_OF_ELEMENTS {
        assert(T::default() == vec.get(i).unwrap().read());
        i += 1;
    }

    // Store empty vector into non-empty storage vector and verify.
    let empty_vec: Vec<T> = Vec::new();
    vec.store_vec(empty_vec);
    assert(0 == vec.len());

    // Store single-element vector into empty storage vector and verify.
    let mut single_vec = Vec::<T>::new();
    single_vec.push(expected_values.get(0).unwrap());
    vec.store_vec(single_vec);
    assert(1 == vec.len());
    assert(expected_values.get(0) == vec.get(0).unwrap().try_read());
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(write)]
    fn assert_store_vec() {
        assert_store_vec_impl::<bool>(1);
        assert_store_vec_impl::<u8>(2);
        assert_store_vec_impl::<u16>(3);
        assert_store_vec_impl::<u32>(4);
        assert_store_vec_impl::<u64>(5);
        assert_store_vec_impl::<u256>(6);
        assert_store_vec_impl::<b256>(7);
        assert_store_vec_impl::<raw_slice>(8);
        assert_store_vec_impl::<str>(9);
        assert_store_vec_impl::<str[2]>(10);
        assert_store_vec_impl::<str[6]>(11);
        assert_store_vec_impl::<str[8]>(12);
        assert_store_vec_impl::<str[12]>(13);
        assert_store_vec_impl::<[u64; 2]>(14);
        assert_store_vec_impl::<ArrayU8Len2>(15);
        assert_store_vec_impl::<ArrayU64Len3>(16);
        assert_store_vec_impl::<ArrayNestedArrayU8Len2Len3>(17);
        assert_store_vec_impl::<ArrayStructBLen2>(18);
        assert_store_vec_impl::<RawPtrNewtype>(19);
        assert_store_vec_impl::<StructA>(20);
        assert_store_vec_impl::<StructB>(21);
        assert_store_vec_impl::<EnumSingleU8>(22);
        assert_store_vec_impl::<EnumSingleU64>(23);
        assert_store_vec_impl::<EnumSingleBool>(24);
        assert_store_vec_impl::<EnumMultiUnits>(25);
        assert_store_vec_impl::<EnumMultiOneByte>(26);
        assert_store_vec_impl::<EnumU8AndU64>(27);
        assert_store_vec_impl::<EnumQuadSlotSize>(28);
        assert_store_vec_impl::<EnumLargerThanQuadSlot>(29);
        assert_store_vec_impl::<(u8, u32)>(30);
    }

    #[storage(write)]
    fn assert_store_vec_nested_storage_type() {
        let vec: StorageKey<StorageVec<StorageVec<u64>>> = StorageKey::new(b256::zero(), 0, b256::zero());
        let heap_vec: Vec<StorageVec<u64>> = Vec::new();
        vec.store_vec(heap_vec); // Nested storage types cannot be stored.
    }

}

#[test]
fn store_vec() {
    let caller = abi(StorageVecStoreVecContractTestsAbi, CONTRACT_ID);
    caller.assert_store_vec();
}

#[test(should_revert)]
fn store_vec_nested_storage_type() {
    let caller = abi(StorageVecStoreVecContractTestsAbi, CONTRACT_ID);
    caller.assert_store_vec_nested_storage_type();
}
