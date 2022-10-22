contract;

abi StorageAccess {
    // Setters
    fn set_s(s: str[40]);
    fn get_s() -> str[40];
}

storage {
    s: str[40] = "0000000000000000000000000000000000000000",
}

impl StorageAccess for Contract {
    fn set_s(s: str[40]) {
        storage.s = s;
    }

    fn get_s() -> str[40] {
        storage.s
    }
}

// check: fn set_s
// check: local mut ptr b256 $(key=$ID)
// check: local mut ptr [b256; 2] $(val_ary=$ID)

// KEY + 0
// check: $(key_ptr=$VAL) = get_ptr mut ptr b256 $key, ptr b256, 0
// check: $(key_val=$VAL) = const b256 0xf383b0ce51358be57daa3b725fe44acdb2d880604e367199080b4379c41bb6ed
// check: store $key_val, ptr $key_ptr

// check: $(val_ary_ptr=$VAL) = get_ptr mut ptr [b256; 2] $val_ary, ptr string<40>, 0
// check: store s, ptr $val_ary_ptr

// check: $(val_ary_0_as_b256=$VAL) = get_ptr mut ptr [b256; 2] $val_ary, ptr b256, 0
// check: state_store_quad_word ptr $val_ary_0_as_b256, key ptr $key_ptr

// KEY + 1
// check: $(key_ptr=$VAL) = get_ptr mut ptr b256 $key, ptr b256, 0
// check: $(key_val=$VAL) = const b256 0xf383b0ce51358be57daa3b725fe44acdb2d880604e367199080b4379c41bb6ee
// check: store $key_val, ptr $key_ptr

// check: $(val_ary_1_as_b256=$VAL) = get_ptr mut ptr [b256; 2] $val_ary, ptr b256, 1
// check: state_store_quad_word ptr $val_ary_1_as_b256, key ptr $key_ptr


// check: fn get_s
// check: local mut ptr b256 $(key=$ID)
// check: local mut ptr [b256; 2] $(val_ary=$ID)

// KEY + 0
// check: $(key_ptr=$VAL) = get_ptr mut ptr b256 $key, ptr b256, 0
// check: $(key_val=$VAL) = const b256 0xf383b0ce51358be57daa3b725fe44acdb2d880604e367199080b4379c41bb6ed
// check: store $key_val, ptr $key_ptr

// check: $(val_ary_as_string=$VAL) = get_ptr mut ptr [b256; 2] $val_ary, ptr string<40>, 0

// check: $(val_ary_0_as_b256=$VAL) = get_ptr mut ptr [b256; 2] $val_ary, ptr b256, 0
// check: state_load_quad_word ptr $val_ary_0_as_b256, key ptr $key_ptr

// KEY + 1
// check: $(key_ptr=$VAL) = get_ptr mut ptr b256 $key, ptr b256, 0
// check: $(key_val=$VAL) = const b256 0xf383b0ce51358be57daa3b725fe44acdb2d880604e367199080b4379c41bb6ee
// check: store $key_val, ptr $key_ptr

// check: $(val_ary_1_as_b256=$VAL) = get_ptr mut ptr [b256; 2] $val_ary, ptr b256, 1
// check: state_load_quad_word ptr $val_ary_1_as_b256, key ptr $key_ptr

// check: ret string<40> $val_ary_as_string
