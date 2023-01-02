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

// check: fn get_e<01665bf4>() -> { { u64, ( { u64, u64, u64, u64, u64 } | u64 ) }, { u64, ( { u64, u64, u64, u64, u64 } | u64 ) } }

// check: local b256 key_for_0_0
// check: local b256 key_for_0_1
// check: local b256 key_for_1_0
// check: local b256 key_for_1_1
// check: local [b256; 2] val_for_0_1
// check: local [b256; 2] val_for_1_1

// check: $(enum_undef=$VAL) = get_local { u64, ( { u64, u64, u64, u64, u64 } | u64 ) } $ID
// check: $(local_key_var=$VAL) = get_local b256 key_for_0_0
// check: $(key=$VAL) = const b256 0xd625ff6d8e88efd7bb3476e748e5d5935618d78bfc7eedf584fe909ce0809fc3
// check: store $key to $local_key_var
// check: $(stored_tag_ptr=$VAL) = state_load_word key $local_key_var
// check: $(stored_tag=$VAL) = bitcast $stored_tag_ptr to u64

// check: insert_value $enum_undef, { u64, ( { u64, u64, u64, u64, u64 } | u64 ) }, $stored_tag, 0

// check: $(local_key_var2=$VAL) = get_local b256 key_for_0_1
// check: $(key2=$VAL) = const b256 0xc4f29cca5a7266ecbc35c82c55dd2b0059a3db4c83a3410653ec33aded8e9840
// check: store $key2 to $local_key_var2

// check: $VAL = get_local [b256; 2] val_for_0_1

// check: $(storage_val_var=$VAL) = get_local [b256; 2] val_for_0_1
// check: $(storage_val_var_as_b256=$VAL) = cast_ptr $storage_val_var, b256, 0
// check: state_load_quad_word $storage_val_var_as_b256, key $local_key_var2
