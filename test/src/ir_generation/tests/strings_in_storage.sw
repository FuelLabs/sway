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

// check: fn get_s
// check: local b256 $(key=$ID)
// check: local [b256; 2] $(val_ary=$ID)

// check: $(key_var=$VAL) = get_local b256 $key
// check: $(key_val=$VAL) = const b256 0xf383b0ce51358be57daa3b725fe44acdb2d880604e367199080b4379c41bb6ed
// check: store $key_val to $key_var

// check: $(val_ary_var=$VAL) = get_local [b256; 2] $val_ary
// check: $(val_ary_as_string=$VAL) = cast_ptr $val_ary_var, string<40>, 0

// check: $(val_ary_0_var=$VAL) = get_local [b256; 2] $val_ary
// check: $(val_ary_0_as_b256=$VAL) = cast_ptr $val_ary_0_var, b256, 0
// check: $(slot_count=$VAL) = const u64 2
// check: state_load_quad_word $val_ary_0_as_b256, key $key_var, $slot_count

// check: ret string<40> $val_ary_as_string

// check: fn set_s
// check: local b256 $(key=$ID)
// check: local [b256; 2] $(val_ary=$ID)

// check: $(key_var=$VAL) = get_local b256 $key
// check: $(key_val=$VAL) = const b256 0xf383b0ce51358be57daa3b725fe44acdb2d880604e367199080b4379c41bb6ed
// check: store $key_val to $key_var

// check: $(val_ary_var=$VAL) = get_local [b256; 2] $val_ary
// check: $(val_ary_var_as_str=$VAL) = cast_ptr $val_ary_var, string<40>, 0
// check: store s to $val_ary_var_as_str

// check: $(val_ary_0_var=$VAL) = get_local [b256; 2] $val_ary
// check: $(val_ary_0_as_b256=$VAL) = cast_ptr $val_ary_0_var, b256, 0
// check: $(slot_count=$VAL) = const u64 2
// check: state_store_quad_word $val_ary_0_as_b256, key $key_var, $slot_count
