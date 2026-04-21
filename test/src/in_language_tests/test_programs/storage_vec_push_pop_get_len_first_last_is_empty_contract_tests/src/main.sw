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
fn assert_push_pop_get_len_first_last_is_empty_impl<T>(
    slot_id_preimage: u64,
)
where
    T: Eq + TestInstance + AbiEncode,
{
    assert_non_zero_sized_type::<T>();

    let vec: StorageKey<StorageVec<T>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));

    let expected_values = T::instances(NUM_OF_ELEMENTS);

    assert(0 == vec.len());
    assert(vec.is_empty());
    assert(vec.first().is_none());
    assert(vec.last().is_none());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        vec.push(expected_values.get(i).unwrap());
        assert(i + 1 == vec.len());
        assert(!vec.is_empty());
        assert(expected_values.get(0) == vec.first().unwrap().try_read());
        assert(expected_values.get(i) == vec.last().unwrap().try_read());
        i += 1;
    }

    assert(NUM_OF_ELEMENTS == vec.len());

    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        assert(expected_values.get(i) == vec.get(i).unwrap().try_read());
        i += 1;
    }

    let mut i = NUM_OF_ELEMENTS;
    while i > 0 {
        i -= 1;
        assert(expected_values.get(i) == vec.pop());
        assert(i == vec.len());
        if i > 0 {
            assert(!vec.is_empty());
            assert(expected_values.get(0) == vec.first().unwrap().try_read());
            assert(expected_values.get(i - 1) == vec.last().unwrap().try_read());
        } else {
            assert(vec.is_empty());
            assert(vec.first().is_none());
            assert(vec.last().is_none());
        }
    }

    assert(None == vec.pop());
    assert(None == vec.pop());

    assert(0 == vec.len());
    assert(vec.is_empty());
    assert(vec.first().is_none());
    assert(vec.last().is_none());
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read, write)]
    fn assert_push_pop_get_len_first_last_is_empty() {
        assert_push_pop_get_len_first_last_is_empty_impl::<bool>(1);
        assert_push_pop_get_len_first_last_is_empty_impl::<u8>(2);
        assert_push_pop_get_len_first_last_is_empty_impl::<u16>(3);
        assert_push_pop_get_len_first_last_is_empty_impl::<u32>(4);
        assert_push_pop_get_len_first_last_is_empty_impl::<u64>(5);
        assert_push_pop_get_len_first_last_is_empty_impl::<u256>(6);
        assert_push_pop_get_len_first_last_is_empty_impl::<b256>(7);
        assert_push_pop_get_len_first_last_is_empty_impl::<raw_slice>(8);
        assert_push_pop_get_len_first_last_is_empty_impl::<str>(9);
        assert_push_pop_get_len_first_last_is_empty_impl::<str[2]>(10);
        assert_push_pop_get_len_first_last_is_empty_impl::<str[6]>(11);
        assert_push_pop_get_len_first_last_is_empty_impl::<str[8]>(12);
        assert_push_pop_get_len_first_last_is_empty_impl::<str[12]>(13);
        assert_push_pop_get_len_first_last_is_empty_impl::<[u64; 2]>(14);
        assert_push_pop_get_len_first_last_is_empty_impl::<ArrayU8Len2>(15);
        assert_push_pop_get_len_first_last_is_empty_impl::<ArrayU64Len3>(16);
        assert_push_pop_get_len_first_last_is_empty_impl::<ArrayNestedArrayU8Len2Len3>(17);
        assert_push_pop_get_len_first_last_is_empty_impl::<ArrayStructBLen2>(18);
        assert_push_pop_get_len_first_last_is_empty_impl::<RawPtrNewtype>(19);
        assert_push_pop_get_len_first_last_is_empty_impl::<StructA>(20);
        assert_push_pop_get_len_first_last_is_empty_impl::<StructB>(21);
        assert_push_pop_get_len_first_last_is_empty_impl::<EnumSingleU8>(22);
        assert_push_pop_get_len_first_last_is_empty_impl::<EnumSingleU64>(23);
        assert_push_pop_get_len_first_last_is_empty_impl::<EnumSingleBool>(24);
        assert_push_pop_get_len_first_last_is_empty_impl::<EnumMultiUnits>(25);
        assert_push_pop_get_len_first_last_is_empty_impl::<EnumMultiOneByte>(26);
        assert_push_pop_get_len_first_last_is_empty_impl::<EnumU8AndU64>(27);
        assert_push_pop_get_len_first_last_is_empty_impl::<EnumQuadSlotSize>(28);
        assert_push_pop_get_len_first_last_is_empty_impl::<EnumLargerThanQuadSlot>(29);
        assert_push_pop_get_len_first_last_is_empty_impl::<(u8, u32)>(30);
    }

}

#[test]
fn push_pop_get_len_first_last_is_empty() {
    let caller = abi(StorageVecPushPopGetLenFirstLastIsEmptyContractTestsAbi, CONTRACT_ID);
    caller.assert_push_pop_get_len_first_last_is_empty();
}
