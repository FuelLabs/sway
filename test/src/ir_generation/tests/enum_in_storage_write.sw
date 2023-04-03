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
    fn set_e(s: S, u: u64);
}

storage {
    e1: E = E::B(0),
    e2: E = E::B(0),
}

impl StorageAccess for Contract {
    fn set_e(s: S, u: u64) {
        storage.e1 = E::A(s);
        storage.e2 = E::B(u);
    }
}

// check: fn set_e<c1c7877c>(s $MD: { u64, u64, u64, u64, u64 }, u $MD: u64) -> ()

// check: local b256 key_for_0_0
// check: local b256 key_for_0_1
// check: local b256 key_for_1_0
// check: local b256 key_for_1_1
// check: local [b256; 2] val_for_0_1
// check: local [b256; 2] val_for_1_1

// At the moment IRgen is a bit inefficient and it initialises the enum, then makes a copy which it
// then stores.

// check: $VAL = get_local ptr { u64, ( { u64, u64, u64, u64, u64 } | u64 ) }, $ID
// check: $(copy_val=$VAL) = get_local ptr { u64, ( { u64, u64, u64, u64, u64 } | u64 ) }, $ID

// check: $(tag_idx=$VAL) = const u64 0
// check: $(tag_ptr=$VAL) = get_elem_ptr $copy_val, ptr u64, $tag_idx
// check: $(enum_tag=$VAL) = load $tag_ptr

// check: $(key_0_0_var=$VAL) = get_local ptr b256, key_for_0_0
// check: $(key_0_0_val=$VAL) = const b256 0xd625ff6d8e88efd7bb3476e748e5d5935618d78bfc7eedf584fe909ce0809fc3
// check: store $key_0_0_val to $key_0_0_var

// check: $(enum_tag_u64=$VAL) = bitcast $enum_tag to u64
// check: state_store_word $enum_tag_u64, key $key_0_0_var

// check: $(val_idx=$VAL) = const u64 1
// check: $(val_ptr=$VAL) = get_elem_ptr $copy_val, ptr ( { u64, u64, u64, u64, u64 } | u64 ), $val_idx
// check: $(enum_val=$VAL) = load $val_ptr

// check: $(key_0_1_var=$VAL) = get_local ptr b256, key_for_0_1
// check: $(key_0_1_val=$VAL) = const b256 0xc4f29cca5a7266ecbc35c82c55dd2b0059a3db4c83a3410653ec33aded8e9840
// check: store $key_0_1_val to $key_0_1_var

// check: $(val_0_1_var=$VAL) = get_local ptr [b256; 2], val_for_0_1
// check: $(cast_val_0_1=$VAL) = cast_ptr $val_0_1_var to ptr ( { u64, u64, u64, u64, u64 } | u64 )
// check: store $enum_val to $cast_val_0_1
// check: $(val_0_1_var_b256=$VAL) = get_local ptr [b256; 2], val_for_0_1
// check: $(cast_val_0_1_var_b256=$VAL) = cast_ptr $val_0_1_var_b256 to ptr b256
// check: state_store_quad_word $cast_val_0_1_var_b256, key $key_0_1_var
