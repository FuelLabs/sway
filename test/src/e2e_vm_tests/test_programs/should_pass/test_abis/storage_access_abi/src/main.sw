library storage_access_abi;

pub struct S {
    x: u64,
    y: u64,
    z: b256,
    t: T,
}

pub struct T {
    x: u64,
    y: u64,
    z: b256,
    boolean: bool,
    int8: u8,
    int16: u16,
    int32: u32,
}

abi StorageAccess {
    // Setters
    fn set_x(x: u64);
    fn set_y(y: b256);
    fn set_s(s: S);
    fn set_boolean(boolean: bool);
    fn set_int8(int8: u8);
    fn set_int16(int16: u16);
    fn set_int32(int32: u32);
    fn set_s_dot_t(t: T);
    fn set_s_dot_x(x: u64);
    fn set_s_dot_y(y: u64);
    fn set_s_dot_z(z: b256);
    fn set_s_dot_t_dot_x(a: u64);
    fn set_s_dot_t_dot_y(b: u64);
    fn set_s_dot_t_dot_z(c: b256);
    fn set_s_dot_t_dot_boolean(boolean: bool);
    fn set_s_dot_t_dot_int8(int8: u8);
    fn set_s_dot_t_dot_int16(int16: u16);
    fn set_s_dot_t_dot_int32(int32: u32);

    // Getters
    fn get_x() -> u64;
    fn get_y() -> b256;
    fn get_s() -> S;
    fn get_boolean() -> bool;
    fn get_int8() -> u8;
    fn get_int16() -> u16;
    fn get_int32() -> u32;
    fn get_s_dot_x() -> u64;
    fn get_s_dot_y() -> u64;
    fn get_s_dot_z() -> b256;
    fn get_s_dot_t() -> T;
    fn get_s_dot_t_dot_x() -> u64;
    fn get_s_dot_t_dot_y() -> u64;
    fn get_s_dot_t_dot_z() -> b256;
    fn get_s_dot_t_dot_boolean() -> bool;
    fn get_s_dot_t_dot_int8() -> u8;
    fn get_s_dot_t_dot_int16() -> u16;
    fn get_s_dot_t_dot_int32() -> u32;
}
