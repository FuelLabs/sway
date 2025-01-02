contract;

use std::storage::storage_vec::*;

abi MyContract {
    #[storage(read, write)]
    fn store(value: Vec<u64>);

    #[storage(read)]
    fn for_iter(value: Vec<u64>) -> bool;

    #[storage(read)]
    fn next_iter(value: Vec<u64>) -> bool;
}

storage {
    my_vec: StorageVec<u64> = StorageVec {},
}

impl MyContract for Contract {
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
                assert_eq(element.read(), value.get(vec_iter).unwrap());
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
