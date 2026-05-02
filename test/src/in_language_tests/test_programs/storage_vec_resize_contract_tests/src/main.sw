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
fn assert_resize_impl<T>(
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

    // Grow and verify appended values are set to `value`.
    let grow_value = expected_values.get(NUM_OF_ELEMENTS / 2).unwrap();
    vec.resize(NUM_OF_ELEMENTS + 2, grow_value);
    assert(NUM_OF_ELEMENTS + 2 == vec.len());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        i += 1;
    }
    assert(grow_value == vec.get(NUM_OF_ELEMENTS).unwrap().read());
    assert(grow_value == vec.get(NUM_OF_ELEMENTS + 1).unwrap().read());

    // Truncate and verify prefix is preserved.
    vec.resize(NUM_OF_ELEMENTS - 2, T::default());
    assert(NUM_OF_ELEMENTS - 2 == vec.len());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS - 2 {
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    // Resize to zero, then grow from empty.
    vec.resize(0, T::default());
    assert(0 == vec.len());

    let from_empty_value = expected_values.get(0).unwrap();
    vec.resize(2, from_empty_value);
    assert(2 == vec.len());
    assert(from_empty_value == vec.get(0).unwrap().read());
    assert(from_empty_value == vec.get(1).unwrap().read());
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read, write)]
    fn assert_resize() {
        assert_resize_impl::<bool>(1);
        assert_resize_impl::<u8>(2);
        assert_resize_impl::<u16>(3);
        assert_resize_impl::<u32>(4);
        assert_resize_impl::<u64>(5);
        assert_resize_impl::<u256>(6);
        assert_resize_impl::<b256>(7);
        assert_resize_impl::<raw_slice>(8);
        assert_resize_impl::<str>(9);
        assert_resize_impl::<str[2]>(10);
        assert_resize_impl::<str[5]>(11);
        assert_resize_impl::<str[6]>(12);
        assert_resize_impl::<str[8]>(13);
        assert_resize_impl::<str[12]>(14);
        assert_resize_impl::<str[13]>(15);
        assert_resize_impl::<[u64; 2]>(16);
        assert_resize_impl::<ArrayU8Len2>(17);
        assert_resize_impl::<ArrayU8Len5>(18);
        assert_resize_impl::<ArrayU8Len6>(19);
        assert_resize_impl::<ArrayU8Len8>(20);
        assert_resize_impl::<ArrayU8Len12>(21);
        assert_resize_impl::<ArrayU8Len13>(22);
        assert_resize_impl::<ArrayU64Len3>(23);
        assert_resize_impl::<ArrayNestedArrayU8Len2Len3>(24);
        assert_resize_impl::<ArrayStructBLen2>(25);
        assert_resize_impl::<RawPtrNewtype>(26);
        assert_resize_impl::<StructA>(27);
        assert_resize_impl::<StructB>(28);
        assert_resize_impl::<EnumSingleU8>(29);
        assert_resize_impl::<EnumSingleU64>(30);
        assert_resize_impl::<EnumSingleBool>(31);
        assert_resize_impl::<EnumMultiUnits>(32);
        assert_resize_impl::<EnumMultiOneByte>(33);
        assert_resize_impl::<EnumU8AndU64>(34);
        assert_resize_impl::<EnumQuadSlotSize>(35);
        assert_resize_impl::<EnumLargerThanQuadSlot>(36);
        assert_resize_impl::<(u8, u32)>(37);
    }

    #[storage(read, write)]
    fn assert_resize_nested_storage_type() {
        let vec: StorageKey<StorageVec<StorageVec<u64>>> = StorageKey::new(b256::zero(), 0, b256::zero());

        // Resize a nested-storage-type vector; only the length is adjusted.
        vec.resize(3, StorageVec {});
        assert(3 == vec.len());

        vec.resize(1, StorageVec {});
        assert(1 == vec.len());

        vec.resize(0, StorageVec {});
        assert(0 == vec.len());
    }

}

#[test]
fn resize() {
    let caller = abi(StorageVecResizeContractTestsAbi, CONTRACT_ID);
    caller.assert_resize();
}

#[test]
fn resize_nested_storage_type() {
    let caller = abi(StorageVecResizeContractTestsAbi, CONTRACT_ID);
    caller.assert_resize_nested_storage_type();
}
