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

// check: $(=^\s*)pub fn set_e<c1c7877c>(s $MD: { u64, u64, u64, u64, u64 }, u $MD: u64) -> ()

// check: local mut ptr b256 key_for_0_0
// check: local mut ptr b256 key_for_0_1
// check: local mut ptr b256 key_for_1_0
// check: local mut ptr b256 key_for_1_1
// check: local mut ptr [b256; 2] val_for_0_1
// check: local mut ptr [b256; 2] val_for_1_1

// check: $(enum_tag=$VAL) = extract_value $VAL, { u64, ( { u64, u64, u64, u64, u64 } | u64 ) }, 0

// check: $(key_0_0_ptr=$VAL) = get_ptr mut ptr b256 key_for_0_0, ptr b256, 0
// check: $(key_0_0_val=$VAL) = const b256 0xd625ff6d8e88efd7bb3476e748e5d5935618d78bfc7eedf584fe909ce0809fc3
// check: store $key_0_0_val, ptr $key_0_0_ptr

// check: $(enum_tag_u64=$VAL) = bitcast $enum_tag to u64
// check: state_store_word $enum_tag_u64, key ptr $key_0_0_ptr

// check: $(enum_val=$VAL) = extract_value $VAL, { u64, ( { u64, u64, u64, u64, u64 } | u64 ) }, 1

// check: $(key_0_1_ptr=$VAL) = get_ptr mut ptr b256 key_for_0_1, ptr b256, 0
// check: $(key_0_1_val=$VAL) = const b256 0xc4f29cca5a7266ecbc35c82c55dd2b0059a3db4c83a3410653ec33aded8e9840
// check: store $key_0_1_val, ptr $key_0_1_ptr

// check: $(val_0_1_ptr=$VAL) = get_ptr mut ptr [b256; 2] val_for_0_1, ptr ( { u64, u64, u64, u64, u64 } | u64 ), 0
// check: store $enum_val, ptr $val_0_1_ptr
// check: $(val_0_1_ptr_b256=$VAL) = get_ptr mut ptr [b256; 2] val_for_0_1, ptr b256, 0
// check: state_store_quad_word ptr $val_0_1_ptr_b256, key ptr $key_0_1_ptr
