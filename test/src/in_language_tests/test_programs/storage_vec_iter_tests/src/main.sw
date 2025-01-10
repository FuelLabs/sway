contract;

// use core::storage::*;
// use std::storage::storage_key::*;
use std::storage::storage_vec::*;
use std::option::*;

abi MyContract {
    #[storage(read, write)]
    fn store(value: Vec<u64>);

    #[storage(read)]
    fn for_iter(value: Vec<u64>) -> bool;

    #[storage(read)]
    fn next_iter(value: Vec<u64>) -> bool;

    #[storage(read, write)]
    fn assert_empty_vec_next_returns_none();
}

storage {
    my_vec: StorageVec<u64> = StorageVec {},
}

fn assert_empty_vec_next_returns_none_impl<T>() where T: AbiEncode + Eq {
    // let vec = StorageKey::<StorageVec<T>>::new(b256::zero(), 0, b256::zero());
    let vec: StorageKey<StorageVec<T>> = StorageKey::new(b256::zero(), 0, b256::zero());
    assert(vec.iter().next().is_none());
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn assert_empty_vec_next_returns_none() {
        assert_empty_vec_next_returns_none_impl::<u64>();
    }

    #[storage(read, write)]
    fn store(value: Vec<u64>) {
        let mut iter = 0;
        while iter < value.len() {
            storage.my_vec.push(value.get(iter).unwrap());
            iter += 1;
        }
    }

    #[storage(read)]
    fn for_iter(value: Vec<u64>) -> bool {
        let mut vec_iter = 0;
        for element in storage.my_vec.iter() {
            if element.read() != value.get(vec_iter).unwrap() {
                return false;
            }

            vec_iter += 1;
        }

        true
    }

    #[storage(read)]
    fn next_iter(value: Vec<u64>) -> bool {
        let mut vec_iter = 0;
        let mut stored_iter = storage.my_vec.iter();

        while vec_iter < value.len() {
            let result = stored_iter.next();
            if result.unwrap().read() != value.get(vec_iter).unwrap() {
                return false;
            }

            vec_iter += 1;
        }

        let none_result = stored_iter.next();
        if none_result.is_some() {
            return false;
        }

        true
    }
}

#[test]
fn empty_vec_next_returns_none() {
    let contract_abi = abi(MyContract, CONTRACT_ID);
    contract_abi.assert_empty_vec_next_returns_none();
}

#[test]
fn for_u64() {
    let contract_abi = abi(MyContract, CONTRACT_ID);

    let mut values = Vec::new();
    let mut i = 0;
    while i < 100 {
        values.push(i);
        i += 1;
    }

    contract_abi.store(values);
    let res = contract_abi.for_iter(values);
    assert(res);
}

#[test]
fn next_u64() {
    let contract_abi = abi(MyContract, CONTRACT_ID);

    let mut values = Vec::new();
    let mut i = 0;
    while i < 100 {
        values.push(i);
        i += 1;
    }

    contract_abi.store(values);
    let res = contract_abi.next_iter(values);
    assert(res);
}
