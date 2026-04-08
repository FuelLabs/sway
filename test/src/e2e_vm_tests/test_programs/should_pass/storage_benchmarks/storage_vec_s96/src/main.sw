contract;

use stored_types::*;
use std::storage::storage_vec::*;

storage {
    vec: StorageVec<Struct96> = StorageVec {},
}

impl Contract {

    // === Baseline (empty contract method call) ===

    fn baseline() { }

    // === Baselines (populate N elements) ===

    #[storage(read, write)]
    fn baseline_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
    }

    #[storage(read, write)]
    fn baseline_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
    }

    #[storage(read, write)]
    fn baseline_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
    }

    // === Baselines (build heap Vec of N elements) ===

    fn baseline_store_vec_n10() {
        let mut v = Vec::<Struct96>::new();
        let mut i = 0;
        while i < 10 {
            v.push(STRUCT96_DEFAULT);
            i += 1;
        }
    }

    fn baseline_store_vec_n100() {
        let mut v = Vec::<Struct96>::new();
        let mut i = 0;
        while i < 100 {
            v.push(STRUCT96_DEFAULT);
            i += 1;
        }
    }

    fn baseline_store_vec_n1000() {
        let mut v = Vec::<Struct96>::new();
        let mut i = 0;
        while i < 1000 {
            v.push(STRUCT96_DEFAULT);
            i += 1;
        }
    }

    // === push ===

    #[storage(read, write)]
    fn push_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.push(STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn push_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.push(STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn push_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.push(STRUCT96_DEFAULT);
    }

    // === push_many ===

    #[storage(read, write)]
    fn push_many_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
    }

    #[storage(read, write)]
    fn push_many_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
    }

    #[storage(read, write)]
    fn push_many_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
    }

    // === pop ===

    #[storage(read, write)]
    fn pop_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.pop();
    }

    #[storage(read, write)]
    fn pop_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.pop();
    }

    #[storage(read, write)]
    fn pop_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.pop();
    }

    // === get ===

    #[storage(read, write)]
    fn get_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.get(5).unwrap().try_read();
    }

    #[storage(read, write)]
    fn get_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.get(50).unwrap().try_read();
    }

    #[storage(read, write)]
    fn get_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.get(500).unwrap().try_read();
    }

    // === set ===

    #[storage(read, write)]
    fn set_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.set(5, STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn set_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.set(50, STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn set_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.set(500, STRUCT96_DEFAULT);
    }

    // === first ===

    #[storage(read, write)]
    fn first_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.first().unwrap().try_read();
    }

    #[storage(read, write)]
    fn first_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.first().unwrap().try_read();
    }

    #[storage(read, write)]
    fn first_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.first().unwrap().try_read();
    }

    // === last ===

    #[storage(read, write)]
    fn last_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.last().unwrap().try_read();
    }

    #[storage(read, write)]
    fn last_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.last().unwrap().try_read();
    }

    #[storage(read, write)]
    fn last_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.last().unwrap().try_read();
    }

    // === len ===

    #[storage(read, write)]
    fn len_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.len();
    }

    #[storage(read, write)]
    fn len_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.len();
    }

    #[storage(read, write)]
    fn len_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.len();
    }

    // === is_empty ===

    #[storage(read, write)]
    fn is_empty_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.is_empty();
    }

    #[storage(read, write)]
    fn is_empty_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.is_empty();
    }

    #[storage(read, write)]
    fn is_empty_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.is_empty();
    }

    // === swap ===

    #[storage(read, write)]
    fn swap_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.swap(0, 9);
    }

    #[storage(read, write)]
    fn swap_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.swap(0, 99);
    }

    #[storage(read, write)]
    fn swap_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.swap(0, 999);
    }

    // === swap_remove ===

    #[storage(read, write)]
    fn swap_remove_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.swap_remove(5);
    }

    #[storage(read, write)]
    fn swap_remove_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.swap_remove(50);
    }

    #[storage(read, write)]
    fn swap_remove_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.swap_remove(500);
    }

    // === remove ===

    #[storage(read, write)]
    fn remove_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.remove(5);
    }

    #[storage(read, write)]
    fn remove_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.remove(50);
    }

    #[storage(read, write)]
    fn remove_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.remove(500);
    }

    // === insert ===

    #[storage(read, write)]
    fn insert_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.insert(5, STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn insert_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.insert(50, STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn insert_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.insert(500, STRUCT96_DEFAULT);
    }

    // === reverse ===

    #[storage(read, write)]
    fn reverse_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.reverse();
    }

    #[storage(read, write)]
    fn reverse_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.reverse();
    }

    #[storage(read, write)]
    fn reverse_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.reverse();
    }

    // === fill ===

    #[storage(read, write)]
    fn fill_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.fill(STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn fill_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.fill(STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn fill_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.fill(STRUCT96_DEFAULT);
    }

    // === resize_grow ===

    #[storage(read, write)]
    fn resize_grow_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.resize(20, STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn resize_grow_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.resize(200, STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn resize_grow_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.resize(2000, STRUCT96_DEFAULT);
    }

    // === resize_shrink ===

    #[storage(read, write)]
    fn resize_shrink_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.resize(5, STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn resize_shrink_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.resize(50, STRUCT96_DEFAULT);
    }

    #[storage(read, write)]
    fn resize_shrink_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.resize(500, STRUCT96_DEFAULT);
    }

    // === store_vec ===

    #[storage(write)]
    fn store_vec_n10() {
        let mut v = Vec::<Struct96>::new();
        let mut i = 0;
        while i < 10 {
            v.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.store_vec(v);
    }

    #[storage(write)]
    fn store_vec_n100() {
        let mut v = Vec::<Struct96>::new();
        let mut i = 0;
        while i < 100 {
            v.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.store_vec(v);
    }

    #[storage(write)]
    fn store_vec_n1000() {
        let mut v = Vec::<Struct96>::new();
        let mut i = 0;
        while i < 1000 {
            v.push(STRUCT96_DEFAULT);
            i += 1;
        }
        storage.vec.store_vec(v);
    }

    // === load_vec ===

    #[storage(read, write)]
    fn load_vec_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.load_vec();
    }

    #[storage(read, write)]
    fn load_vec_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.load_vec();
    }

    #[storage(read, write)]
    fn load_vec_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.load_vec();
    }

    // === iter ===

    #[storage(read, write)]
    fn iter_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        for elem in storage.vec.iter() {
            let _ = elem.try_read();
        }
    }

    #[storage(read, write)]
    fn iter_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        for elem in storage.vec.iter() {
            let _ = elem.try_read();
        }
    }

    #[storage(read, write)]
    fn iter_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        for elem in storage.vec.iter() {
            let _ = elem.try_read();
        }
    }

    // === clear ===

    #[storage(read, write)]
    fn clear_n10() {
        let mut i = 0;
        while i < 10 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.clear();
    }

    #[storage(read, write)]
    fn clear_n100() {
        let mut i = 0;
        while i < 100 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.clear();
    }

    #[storage(read, write)]
    fn clear_n1000() {
        let mut i = 0;
        while i < 1000 {
            storage.vec.push(STRUCT96_DEFAULT);
            i += 1;
        }
        let _ = storage.vec.clear();
    }
}

// === Baseline test (empty call) ===

#[test]
fn bench_baseline() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.baseline();
}

// === Baseline tests (populate) ===

#[test]
fn bench_baseline_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.baseline_n10();
}

#[test]
fn bench_baseline_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.baseline_n100();
}

#[test]
fn bench_baseline_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.baseline_n1000();
}

// === Baseline tests (store_vec) ===

#[test]
fn bench_baseline_store_vec_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.baseline_store_vec_n10();
}

#[test]
fn bench_baseline_store_vec_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.baseline_store_vec_n100();
}

#[test]
fn bench_baseline_store_vec_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.baseline_store_vec_n1000();
}

// === push tests ===

#[test]
fn bench_push_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.push_n10();
}

#[test]
fn bench_push_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.push_n100();
}

#[test]
fn bench_push_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.push_n1000();
}

// === push_many tests ===

#[test]
fn bench_push_many_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.push_many_n10();
}

#[test]
fn bench_push_many_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.push_many_n100();
}

#[test]
fn bench_push_many_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.push_many_n1000();
}

// === pop tests ===

#[test]
fn bench_pop_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.pop_n10();
}

#[test]
fn bench_pop_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.pop_n100();
}

#[test]
fn bench_pop_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.pop_n1000();
}

// === get tests ===

#[test]
fn bench_get_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.get_n10();
}

#[test]
fn bench_get_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.get_n100();
}

#[test]
fn bench_get_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.get_n1000();
}

// === set tests ===

#[test]
fn bench_set_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.set_n10();
}

#[test]
fn bench_set_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.set_n100();
}

#[test]
fn bench_set_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.set_n1000();
}

// === first tests ===

#[test]
fn bench_first_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.first_n10();
}

#[test]
fn bench_first_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.first_n100();
}

#[test]
fn bench_first_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.first_n1000();
}

// === last tests ===

#[test]
fn bench_last_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.last_n10();
}

#[test]
fn bench_last_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.last_n100();
}

#[test]
fn bench_last_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.last_n1000();
}

// === len tests ===

#[test]
fn bench_len_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.len_n10();
}

#[test]
fn bench_len_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.len_n100();
}

#[test]
fn bench_len_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.len_n1000();
}

// === is_empty tests ===

#[test]
fn bench_is_empty_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.is_empty_n10();
}

#[test]
fn bench_is_empty_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.is_empty_n100();
}

#[test]
fn bench_is_empty_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.is_empty_n1000();
}

// === swap tests ===

#[test]
fn bench_swap_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.swap_n10();
}

#[test]
fn bench_swap_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.swap_n100();
}

#[test]
fn bench_swap_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.swap_n1000();
}

// === swap_remove tests ===

#[test]
fn bench_swap_remove_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.swap_remove_n10();
}

#[test]
fn bench_swap_remove_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.swap_remove_n100();
}

#[test]
fn bench_swap_remove_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.swap_remove_n1000();
}

// === remove tests ===

#[test]
fn bench_remove_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.remove_n10();
}

#[test]
fn bench_remove_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.remove_n100();
}

#[test]
fn bench_remove_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.remove_n1000();
}

// === insert tests ===

#[test]
fn bench_insert_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.insert_n10();
}

#[test]
fn bench_insert_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.insert_n100();
}

#[test]
fn bench_insert_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.insert_n1000();
}

// === reverse tests ===

#[test]
fn bench_reverse_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.reverse_n10();
}

#[test]
fn bench_reverse_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.reverse_n100();
}

#[test]
fn bench_reverse_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.reverse_n1000();
}

// === fill tests ===

#[test]
fn bench_fill_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.fill_n10();
}

#[test]
fn bench_fill_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.fill_n100();
}

#[test]
fn bench_fill_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.fill_n1000();
}

// === resize_grow tests ===

#[test]
fn bench_resize_grow_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.resize_grow_n10();
}

#[test]
fn bench_resize_grow_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.resize_grow_n100();
}

#[test]
fn bench_resize_grow_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.resize_grow_n1000();
}

// === resize_shrink tests ===

#[test]
fn bench_resize_shrink_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.resize_shrink_n10();
}

#[test]
fn bench_resize_shrink_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.resize_shrink_n100();
}

#[test]
fn bench_resize_shrink_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.resize_shrink_n1000();
}

// === store_vec tests ===

#[test]
fn bench_store_vec_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.store_vec_n10();
}

#[test]
fn bench_store_vec_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.store_vec_n100();
}

#[test]
fn bench_store_vec_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.store_vec_n1000();
}

// === load_vec tests ===

#[test]
fn bench_load_vec_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.load_vec_n10();
}

#[test]
fn bench_load_vec_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.load_vec_n100();
}

#[test]
fn bench_load_vec_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.load_vec_n1000();
}

// === iter tests ===

#[test]
fn bench_iter_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.iter_n10();
}

#[test]
fn bench_iter_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.iter_n100();
}

#[test]
fn bench_iter_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.iter_n1000();
}

// === clear tests ===

#[test]
fn bench_clear_n10() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.clear_n10();
}

#[test]
fn bench_clear_n100() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.clear_n100();
}

#[test]
fn bench_clear_n1000() {
    let caller = abi(StorageVecS96Abi, CONTRACT_ID);
    caller.clear_n1000();
}

