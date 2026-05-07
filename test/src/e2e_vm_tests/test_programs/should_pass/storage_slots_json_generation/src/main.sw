contract;

storage {
    f_bool: bool = true,
    f_u8: u8 = 2,
    f_u16: u16 = 3,
    f_u32: u32 = 4,
    f_u64: u64 = 5,
    f_u256: u256 = 6,
    f_b256: b256 = 0xABCD1234ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234,
    f_str_2: str[2] = __to_str_array("aa"),
    f_str_5: str[5] = __to_str_array("aaaaa"),
    f_str_32: str[32] = __to_str_array("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    f_str_33: str[33] = __to_str_array("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    f_tuple_4: (u8, bool, str[3]) = (7, false, __to_str_array("abc")),
    f_tuple_41: (u64, bool, str[32]) = (7, false, __to_str_array("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),

    n1 {
        f_bool: bool = true,
        f_u8: u8 = 2,
        f_u16: u16 = 3,
        f_u32: u32 = 4,
        f_u64: u64 = 5,
        f_u256: u256 = 6,
        f_b256: b256 = 0xABCD1234ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234,
        f_str_2: str[2] = __to_str_array("aa"),
        f_str_5: str[5] = __to_str_array("aaaaa"),
        f_str_32: str[32] = __to_str_array("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        f_str_33: str[33] = __to_str_array("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        f_tuple_4: (u8, bool, str[3]) = (7, false, __to_str_array("abc")),
        f_tuple_41: (u64, bool, str[32]) = (7, false, __to_str_array("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),

        n2 {
            f_bool: bool = true,
            f_u8: u8 = 2,
            f_u16: u16 = 3,
            f_u32: u32 = 4,
            f_u64: u64 = 5,
            f_u256: u256 = 6,
            f_b256: b256 = 0xABCD1234ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234,
            f_str_2: str[2] = __to_str_array("aa"),
            f_str_5: str[5] = __to_str_array("aaaaa"),
            f_str_32: str[32] = __to_str_array("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            f_str_33: str[33] = __to_str_array("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            f_tuple_4: (u8, bool, str[3]) = (7, false, __to_str_array("abc")),
            f_tuple_41: (u64, bool, str[32]) = (7, false, __to_str_array("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        }
    }
}

impl Contract { }
