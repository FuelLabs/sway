contract;

struct S {
    x: u64,
    y: u64,
    z: u64,
    w: u64,
    b: u64,
}

pub enum E {
    A: S,
    B: u64,
}

abi StorageAccess {
    fn get_e() -> (E, E);
}

storage {
    e1: E = E::B(0),
    e2: E = E::B(0),
}

impl StorageAccess for Contract {
    fn get_e() -> (E, E) {
        (storage.e1, storage.e2)
    }
}

// check: $(=^\s*)pub fn get_e<01665bf4>() -> { { u64, ( { u64, u64, u64, u64, u64 } | u64 ) }, { u64, ( { u64, u64, u64, u64, u64 } | u64 ) } }

// check: local mut ptr b256 key_for_0_0
// check: local mut ptr b256 key_for_0_1
// check: local mut ptr b256 key_for_1_0
// check: local mut ptr b256 key_for_1_1
// check: local mut ptr [b256; 2] val_for_0_1
// check: local mut ptr [b256; 2] val_for_1_1

// check: $(enum_undef=$VAL) = get_ptr ptr { u64, ( { u64, u64, u64, u64, u64 } | u64 ) } $ID, ptr { u64, ( { u64, u64, u64, u64, u64 } | u64 ) }, 0
// check: $(local_key_ptr=$VAL) = get_ptr mut ptr b256 key_for_0_0, ptr b256, 0
// check: $(key=$VAL) = const b256 0xd625ff6d8e88efd7bb3476e748e5d5935618d78bfc7eedf584fe909ce0809fc3
// check: store $key, ptr $local_key_ptr
// check: $(stored_tag_ptr=$VAL) = state_load_word key ptr $local_key_ptr
// check: $(stored_tag=$VAL) = bitcast $stored_tag_ptr to u64

// check: insert_value $enum_undef, { u64, ( { u64, u64, u64, u64, u64 } | u64 ) }, $stored_tag, 0

// check: $(local_key_ptr2=$VAL) = get_ptr mut ptr b256 key_for_0_1, ptr b256, 0
// check: $(key2=$VAL) = const b256 0xc4f29cca5a7266ecbc35c82c55dd2b0059a3db4c83a3410653ec33aded8e9840
// check: store $key2, ptr $local_key_ptr2
// check: $(storage_val_ptr2=$VAL) = get_ptr mut ptr [b256; 2] val_for_0_1, ptr b256, 0
// check: state_load_quad_word ptr $storage_val_ptr2, key ptr $local_key_ptr2
