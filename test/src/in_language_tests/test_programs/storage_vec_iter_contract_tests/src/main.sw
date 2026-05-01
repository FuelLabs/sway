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

storage {
    vec: StorageVec<u64> = StorageVec {},
    vec_of_vec: StorageVec<StorageVec<u64>> = StorageVec {},
}

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[storage(read)]
fn assert_empty_vec_next_returns_none_impl<T>(slot_id_preimage: u64) {
    let vec: StorageKey<StorageVec<T>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));
    assert(vec.iter().next().is_none());
}

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[storage(read, write)]
fn assert_vec_with_elements_next_returns_element_impl<T>(
    slot_id_preimage: u64,
)
where
    T: Eq + TestInstance + AbiEncode,
{
    let vec: StorageKey<StorageVec<T>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));

    const NUM_OF_ELEMENTS: u64 = 37;
    let expected_values = T::instances(NUM_OF_ELEMENTS);
    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        vec.push(expected_values.get(i).unwrap());
        i += 1;
    }

    let mut i = 0;
    let mut iter = vec.iter();
    while i < NUM_OF_ELEMENTS {
        let element = iter.next();
        assert(element.is_some());

        let value = element.unwrap().read();
        assert(value == vec.get(i).unwrap().read());

        i += 1;
    }

    let element_after_last = iter.next();
    assert(element_after_last.is_none());
    let element_after_last = iter.next();
    assert(element_after_last.is_none());
}

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[storage(read, write)]
fn assert_vec_with_elements_for_loop_iteration_impl<T>(
    slot_id_preimage: u64,
)
where
    T: Eq + TestInstance + AbiEncode,
{
    let vec: StorageKey<StorageVec<T>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));

    const NUM_OF_ELEMENTS: u64 = 37;
    let expected_values = T::instances(NUM_OF_ELEMENTS);
    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        vec.push(expected_values.get(i).unwrap());
        i += 1;
    }

    let mut i = 0;
    for element in vec.iter() {
        let value = element.read();
        assert(value == vec.get(i).unwrap().read());

        i += 1;
    }

    assert(vec.len() == i);
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read)]
    fn assert_empty_vec_next_returns_none() {
        assert_empty_vec_next_returns_none_impl::<bool>(1);
        assert_empty_vec_next_returns_none_impl::<u8>(2);
        assert_empty_vec_next_returns_none_impl::<u16>(3);
        assert_empty_vec_next_returns_none_impl::<u32>(4);
        assert_empty_vec_next_returns_none_impl::<u64>(5);
        assert_empty_vec_next_returns_none_impl::<u256>(6);
        assert_empty_vec_next_returns_none_impl::<b256>(7);
        assert_empty_vec_next_returns_none_impl::<raw_slice>(8);
        assert_empty_vec_next_returns_none_impl::<str>(9);
        assert_empty_vec_next_returns_none_impl::<str[2]>(10);
        assert_empty_vec_next_returns_none_impl::<str[5]>(11);
        assert_empty_vec_next_returns_none_impl::<str[6]>(12);
        assert_empty_vec_next_returns_none_impl::<str[8]>(13);
        assert_empty_vec_next_returns_none_impl::<str[12]>(14);
        assert_empty_vec_next_returns_none_impl::<str[13]>(15);
        assert_empty_vec_next_returns_none_impl::<[u64; 2]>(16);
        assert_empty_vec_next_returns_none_impl::<ArrayU8Len2>(17);
        assert_empty_vec_next_returns_none_impl::<ArrayU8Len5>(18);
        assert_empty_vec_next_returns_none_impl::<ArrayU8Len6>(19);
        assert_empty_vec_next_returns_none_impl::<ArrayU8Len8>(20);
        assert_empty_vec_next_returns_none_impl::<ArrayU8Len12>(21);
        assert_empty_vec_next_returns_none_impl::<ArrayU8Len13>(22);
        assert_empty_vec_next_returns_none_impl::<ArrayU64Len3>(23);
        assert_empty_vec_next_returns_none_impl::<ArrayNestedArrayU8Len2Len3>(24);
        assert_empty_vec_next_returns_none_impl::<ArrayStructBLen2>(25);
        assert_empty_vec_next_returns_none_impl::<RawPtrNewtype>(26);
        assert_empty_vec_next_returns_none_impl::<StructA>(27);
        assert_empty_vec_next_returns_none_impl::<StructB>(28);
        assert_empty_vec_next_returns_none_impl::<EnumSingleU8>(29);
        assert_empty_vec_next_returns_none_impl::<EnumSingleU64>(30);
        assert_empty_vec_next_returns_none_impl::<EnumSingleBool>(31);
        assert_empty_vec_next_returns_none_impl::<EnumMultiUnits>(32);
        assert_empty_vec_next_returns_none_impl::<EnumMultiOneByte>(33);
        assert_empty_vec_next_returns_none_impl::<EnumU8AndU64>(34);
        assert_empty_vec_next_returns_none_impl::<EnumQuadSlotSize>(35);
        assert_empty_vec_next_returns_none_impl::<EnumLargerThanQuadSlot>(36);
        assert_empty_vec_next_returns_none_impl::<(u8, u32)>(37);
    }

    #[storage(read, write)]
    fn assert_vec_with_elements_next_returns_element() {
        assert_vec_with_elements_next_returns_element_impl::<bool>(1);
        assert_vec_with_elements_next_returns_element_impl::<u8>(2);
        assert_vec_with_elements_next_returns_element_impl::<u16>(3);
        assert_vec_with_elements_next_returns_element_impl::<u32>(4);
        assert_vec_with_elements_next_returns_element_impl::<u64>(5);
        assert_vec_with_elements_next_returns_element_impl::<u256>(6);
        assert_vec_with_elements_next_returns_element_impl::<b256>(7);
        assert_vec_with_elements_next_returns_element_impl::<raw_slice>(8);
        assert_vec_with_elements_next_returns_element_impl::<str>(9);
        assert_vec_with_elements_next_returns_element_impl::<str[2]>(10);
        assert_vec_with_elements_next_returns_element_impl::<str[5]>(11);
        assert_vec_with_elements_next_returns_element_impl::<str[6]>(12);
        assert_vec_with_elements_next_returns_element_impl::<str[8]>(13);
        assert_vec_with_elements_next_returns_element_impl::<str[12]>(14);
        assert_vec_with_elements_next_returns_element_impl::<str[13]>(15);
        assert_vec_with_elements_next_returns_element_impl::<[u64; 2]>(16);
        assert_vec_with_elements_next_returns_element_impl::<ArrayU8Len2>(17);
        assert_vec_with_elements_next_returns_element_impl::<ArrayU8Len5>(18);
        assert_vec_with_elements_next_returns_element_impl::<ArrayU8Len6>(19);
        assert_vec_with_elements_next_returns_element_impl::<ArrayU8Len8>(20);
        assert_vec_with_elements_next_returns_element_impl::<ArrayU8Len12>(21);
        assert_vec_with_elements_next_returns_element_impl::<ArrayU8Len13>(22);
        assert_vec_with_elements_next_returns_element_impl::<ArrayU64Len3>(23);
        assert_vec_with_elements_next_returns_element_impl::<ArrayNestedArrayU8Len2Len3>(24);
        assert_vec_with_elements_next_returns_element_impl::<ArrayStructBLen2>(25);
        assert_vec_with_elements_next_returns_element_impl::<RawPtrNewtype>(26);
        assert_vec_with_elements_next_returns_element_impl::<StructA>(27);
        assert_vec_with_elements_next_returns_element_impl::<StructB>(28);
        assert_vec_with_elements_next_returns_element_impl::<EnumSingleU8>(29);
        assert_vec_with_elements_next_returns_element_impl::<EnumSingleU64>(30);
        assert_vec_with_elements_next_returns_element_impl::<EnumSingleBool>(31);
        assert_vec_with_elements_next_returns_element_impl::<EnumMultiUnits>(32);
        assert_vec_with_elements_next_returns_element_impl::<EnumMultiOneByte>(33);
        assert_vec_with_elements_next_returns_element_impl::<EnumU8AndU64>(34);
        assert_vec_with_elements_next_returns_element_impl::<EnumQuadSlotSize>(35);
        assert_vec_with_elements_next_returns_element_impl::<EnumLargerThanQuadSlot>(36);
        assert_vec_with_elements_next_returns_element_impl::<(u8, u32)>(37);
    }

    #[storage(read, write)]
    fn assert_vec_with_elements_for_loop_iteration() {
        assert_vec_with_elements_for_loop_iteration_impl::<bool>(1);
        assert_vec_with_elements_for_loop_iteration_impl::<u8>(2);
        assert_vec_with_elements_for_loop_iteration_impl::<u16>(3);
        assert_vec_with_elements_for_loop_iteration_impl::<u32>(4);
        assert_vec_with_elements_for_loop_iteration_impl::<u64>(5);
        assert_vec_with_elements_for_loop_iteration_impl::<u256>(6);
        assert_vec_with_elements_for_loop_iteration_impl::<b256>(7);
        assert_vec_with_elements_for_loop_iteration_impl::<raw_slice>(8);
        assert_vec_with_elements_for_loop_iteration_impl::<str>(9);
        assert_vec_with_elements_for_loop_iteration_impl::<str[2]>(10);
        assert_vec_with_elements_for_loop_iteration_impl::<str[5]>(11);
        assert_vec_with_elements_for_loop_iteration_impl::<str[6]>(12);
        assert_vec_with_elements_for_loop_iteration_impl::<str[8]>(13);
        assert_vec_with_elements_for_loop_iteration_impl::<str[12]>(14);
        assert_vec_with_elements_for_loop_iteration_impl::<str[13]>(15);
        assert_vec_with_elements_for_loop_iteration_impl::<[u64; 2]>(16);
        assert_vec_with_elements_for_loop_iteration_impl::<ArrayU8Len2>(17);
        assert_vec_with_elements_for_loop_iteration_impl::<ArrayU8Len5>(18);
        assert_vec_with_elements_for_loop_iteration_impl::<ArrayU8Len6>(19);
        assert_vec_with_elements_for_loop_iteration_impl::<ArrayU8Len8>(20);
        assert_vec_with_elements_for_loop_iteration_impl::<ArrayU8Len12>(21);
        assert_vec_with_elements_for_loop_iteration_impl::<ArrayU8Len13>(22);
        assert_vec_with_elements_for_loop_iteration_impl::<ArrayU64Len3>(23);
        assert_vec_with_elements_for_loop_iteration_impl::<ArrayNestedArrayU8Len2Len3>(24);
        assert_vec_with_elements_for_loop_iteration_impl::<ArrayStructBLen2>(25);
        assert_vec_with_elements_for_loop_iteration_impl::<RawPtrNewtype>(26);
        assert_vec_with_elements_for_loop_iteration_impl::<StructA>(27);
        assert_vec_with_elements_for_loop_iteration_impl::<StructB>(28);
        assert_vec_with_elements_for_loop_iteration_impl::<EnumSingleU8>(29);
        assert_vec_with_elements_for_loop_iteration_impl::<EnumSingleU64>(30);
        assert_vec_with_elements_for_loop_iteration_impl::<EnumSingleBool>(31);
        assert_vec_with_elements_for_loop_iteration_impl::<EnumMultiUnits>(32);
        assert_vec_with_elements_for_loop_iteration_impl::<EnumMultiOneByte>(33);
        assert_vec_with_elements_for_loop_iteration_impl::<EnumU8AndU64>(34);
        assert_vec_with_elements_for_loop_iteration_impl::<EnumQuadSlotSize>(35);
        assert_vec_with_elements_for_loop_iteration_impl::<EnumLargerThanQuadSlot>(36);
        assert_vec_with_elements_for_loop_iteration_impl::<(u8, u32)>(37);
    }

    #[storage(read, write)]
    fn storage_vec_field_for_loop_iteration() {
        const NUM_OF_ELEMENTS: u64 = 37;
        let expected_values = u64::instances(NUM_OF_ELEMENTS);
        let mut i = 0;
        while i < NUM_OF_ELEMENTS {
            storage.vec.push(expected_values.get(i).unwrap());
            i += 1;
        }

        let mut i = 0;
        for element in storage.vec.iter() {
            let value = element.read();
            assert(value == storage.vec.get(i).unwrap().read());

            i += 1;
        }

        assert(storage.vec.len() == i);
    }

    #[storage(read, write)]
    fn storage_vec_field_nested_for_loop_iteration() {
        const NUM_OF_NESTED_VECTORS: u64 = 7;
        const NUM_OF_ELEMENTS: u64 = 37;
        let expected_values = u64::instances(NUM_OF_ELEMENTS);

        let mut i = 0;
        while i < NUM_OF_NESTED_VECTORS {
            let nested_vector = StorageVec::<u64> {};
            storage.vec_of_vec.push(nested_vector);

            let nested_vector = storage.vec_of_vec.get(i).unwrap();

            let mut j = 0;
            while j < NUM_OF_ELEMENTS {
                nested_vector.push(expected_values.get(j).unwrap());
                j += 1;
            }

            i += 1;
        }

        let mut i = 0;
        for nested_vec in storage.vec_of_vec.iter() {
            let mut j = 0;
            for element in nested_vec.iter() {
                let value = element.read();
                let stored_value = storage.vec_of_vec.get(i).unwrap().get(j).unwrap().read();
                assert(value == stored_value);

                j += 1;
            }

            assert(nested_vec.len() == j);

            i += 1;
        }

        assert(storage.vec_of_vec.len() == i);
    }
}

#[test]
fn empty_vec_next_returns_none() {
    let contract_abi = abi(StorageVecIterContractTestsAbi, CONTRACT_ID);
    contract_abi.assert_empty_vec_next_returns_none();
}

#[test]
fn vec_with_elements_next_returns_element() {
    let contract_abi = abi(StorageVecIterContractTestsAbi, CONTRACT_ID);
    contract_abi.assert_vec_with_elements_next_returns_element();
}

#[test]
fn vec_with_elements_for_loop_iteration() {
    let contract_abi = abi(StorageVecIterContractTestsAbi, CONTRACT_ID);
    contract_abi.assert_vec_with_elements_for_loop_iteration();
}

#[test]
fn storage_vec_field_for_loop_iteration() {
    let contract_abi = abi(StorageVecIterContractTestsAbi, CONTRACT_ID);
    contract_abi.storage_vec_field_for_loop_iteration();
}

#[test]
fn storage_vec_field_nested_for_loop_iteration() {
    let contract_abi = abi(StorageVecIterContractTestsAbi, CONTRACT_ID);
    contract_abi.storage_vec_field_nested_for_loop_iteration();
}
