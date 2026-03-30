contract;

mod impls;

use impls::*;
use impls::Enum;

use std::hash::{Hash, sha256};
use std::storage::storage_vec::*;

#[cfg(experimental_dynamic_storage = false)]
storage {
    vec: StorageVec<u64> = StorageVec {},
    vec_of_vec: StorageVec<StorageVec<u64>> = StorageVec {},
}

#[cfg(experimental_dynamic_storage = false)]
#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[storage(read)]
fn assert_empty_vec_next_returns_none_impl<T>(slot_id_preimage: u64) {
    let vec: StorageKey<StorageVec<T>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));
    assert(vec.iter().next().is_none());
}

#[cfg(experimental_dynamic_storage = false)]
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
    let expected_values = T::elements(NUM_OF_ELEMENTS);
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
        assert_eq(value, vec.get(i).unwrap().read());

        i += 1;
    }

    let element_after_last = iter.next();
    assert(element_after_last.is_none());
    let element_after_last = iter.next();
    assert(element_after_last.is_none());
}

#[cfg(experimental_dynamic_storage = false)]
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
    let expected_values = T::elements(NUM_OF_ELEMENTS);
    let mut i = 0;
    while i < NUM_OF_ELEMENTS {
        vec.push(expected_values.get(i).unwrap());
        i += 1;
    }

    let mut i = 0;
    for element in vec.iter() {
        let value = element.read();
        assert_eq(value, vec.get(i).unwrap().read());

        i += 1;
    }

    assert_eq(vec.len(), i);
}

#[cfg(experimental_dynamic_storage = false)]
impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read)]
    fn assert_empty_vec_next_returns_none() {
        assert_empty_vec_next_returns_none_impl::<bool>(2);
        assert_empty_vec_next_returns_none_impl::<u8>(3);
        assert_empty_vec_next_returns_none_impl::<u16>(4);
        assert_empty_vec_next_returns_none_impl::<u32>(5);
        assert_empty_vec_next_returns_none_impl::<u64>(6);
        assert_empty_vec_next_returns_none_impl::<u256>(7);
        assert_empty_vec_next_returns_none_impl::<[u64; 2]>(8);
        assert_empty_vec_next_returns_none_impl::<Struct>(10);
        assert_empty_vec_next_returns_none_impl::<str>(12);
        assert_empty_vec_next_returns_none_impl::<str[6]>(13);
        assert_empty_vec_next_returns_none_impl::<Enum>(14);
        assert_empty_vec_next_returns_none_impl::<(u8, u32)>(15);
        assert_empty_vec_next_returns_none_impl::<b256>(16);
        assert_empty_vec_next_returns_none_impl::<RawPtrNewtype>(17);
        assert_empty_vec_next_returns_none_impl::<raw_slice>(18);
    }

    #[storage(read, write)]
    fn assert_vec_with_elements_next_returns_element() {
        assert_vec_with_elements_next_returns_element_impl::<bool>(2);
        assert_vec_with_elements_next_returns_element_impl::<u8>(3);
        assert_vec_with_elements_next_returns_element_impl::<u16>(4);
        assert_vec_with_elements_next_returns_element_impl::<u32>(5);
        assert_vec_with_elements_next_returns_element_impl::<u64>(6);
        assert_vec_with_elements_next_returns_element_impl::<u256>(7);
        assert_vec_with_elements_next_returns_element_impl::<[u64; 2]>(8);
        assert_vec_with_elements_next_returns_element_impl::<Struct>(10);
        assert_vec_with_elements_next_returns_element_impl::<str>(12);
        assert_vec_with_elements_next_returns_element_impl::<str[6]>(13);
        assert_vec_with_elements_next_returns_element_impl::<Enum>(14);
        assert_vec_with_elements_next_returns_element_impl::<(u8, u32)>(15);
        assert_vec_with_elements_next_returns_element_impl::<b256>(16);
        assert_vec_with_elements_next_returns_element_impl::<RawPtrNewtype>(17);
        assert_vec_with_elements_next_returns_element_impl::<raw_slice>(18);
    }

    #[storage(read, write)]
    fn assert_vec_with_elements_for_loop_iteration() {
        assert_vec_with_elements_for_loop_iteration_impl::<bool>(2);
        assert_vec_with_elements_for_loop_iteration_impl::<u8>(3);
        assert_vec_with_elements_for_loop_iteration_impl::<u16>(4);
        assert_vec_with_elements_for_loop_iteration_impl::<u32>(5);
        assert_vec_with_elements_for_loop_iteration_impl::<u64>(6);
        assert_vec_with_elements_for_loop_iteration_impl::<u256>(7);
        assert_vec_with_elements_for_loop_iteration_impl::<[u64; 2]>(8);
        assert_vec_with_elements_for_loop_iteration_impl::<Struct>(10);
        assert_vec_with_elements_for_loop_iteration_impl::<str>(12);
        assert_vec_with_elements_for_loop_iteration_impl::<str[6]>(13);
        assert_vec_with_elements_for_loop_iteration_impl::<Enum>(14);
        assert_vec_with_elements_for_loop_iteration_impl::<(u8, u32)>(15);
        assert_vec_with_elements_for_loop_iteration_impl::<b256>(16);
        assert_vec_with_elements_for_loop_iteration_impl::<RawPtrNewtype>(17);
        assert_vec_with_elements_for_loop_iteration_impl::<raw_slice>(18);
    }

    #[storage(read, write)]
    fn storage_vec_field_for_loop_iteration() {
        const NUM_OF_ELEMENTS: u64 = 37;
        let expected_values = u64::elements(NUM_OF_ELEMENTS);
        let mut i = 0;
        while i < NUM_OF_ELEMENTS {
            storage.vec.push(expected_values.get(i).unwrap());
            i += 1;
        }

        let mut i = 0;
        for element in storage.vec.iter() {
            let value = element.read();
            assert_eq(value, storage.vec.get(i).unwrap().read());

            i += 1;
        }

        assert_eq(storage.vec.len(), i);
    }

    #[storage(read, write)]
    fn storage_vec_field_nested_for_loop_iteration() {
        const NUM_OF_NESTED_VECTORS: u64 = 7;
        const NUM_OF_ELEMENTS: u64 = 37;
        let expected_values = u64::elements(NUM_OF_ELEMENTS);

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
                assert_eq(value, stored_value);

                j += 1;
            }

            assert_eq(nested_vec.len(), j);

            i += 1;
        }

        assert_eq(storage.vec_of_vec.len(), i);
    }
}

#[cfg(experimental_dynamic_storage = false)]
#[test]
fn empty_vec_next_returns_none() {
    let contract_abi = abi(StorageVecIterTestsAbi, CONTRACT_ID);
    contract_abi.assert_empty_vec_next_returns_none();
}

#[cfg(experimental_dynamic_storage = false)]
#[test]
fn vec_with_elements_next_returns_element() {
    let contract_abi = abi(StorageVecIterTestsAbi, CONTRACT_ID);
    contract_abi.assert_vec_with_elements_next_returns_element();
}

#[cfg(experimental_dynamic_storage = false)]
#[test]
fn vec_with_elements_for_loop_iteration() {
    let contract_abi = abi(StorageVecIterTestsAbi, CONTRACT_ID);
    contract_abi.assert_vec_with_elements_for_loop_iteration();
}

#[cfg(experimental_dynamic_storage = false)]
#[test]
fn storage_vec_field_for_loop_iteration() {
    let contract_abi = abi(StorageVecIterTestsAbi, CONTRACT_ID);
    contract_abi.storage_vec_field_for_loop_iteration();
}

#[cfg(experimental_dynamic_storage = false)]
#[test]
fn storage_vec_field_nested_for_loop_iteration() {
    let contract_abi = abi(StorageVecIterTestsAbi, CONTRACT_ID);
    contract_abi.storage_vec_field_nested_for_loop_iteration();
}
