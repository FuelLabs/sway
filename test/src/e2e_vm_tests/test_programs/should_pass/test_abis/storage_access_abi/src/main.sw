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

pub enum E {
    A: u64,
    B: T,
}

abi StorageAccess {
    // Setters
    #[storage(write)]
    fn set_x(x: u64);
    #[storage(write)]
    fn set_y(y: b256);
    #[storage(write)]
    fn set_s(s: S);
    #[storage(write)]
    fn set_boolean(boolean: bool);
    #[storage(write)]
    fn set_int8(int8: u8);
    #[storage(write)]
    fn set_int16(int16: u16);
    #[storage(write)]
    fn set_int32(int32: u32);
    #[storage(write)]
    fn set_s_dot_t(t: T);
    #[storage(write)]
    fn set_s_dot_x(x: u64);
    #[storage(write)]
    fn set_s_dot_y(y: u64);
    #[storage(write)]
    fn set_s_dot_z(z: b256);
    #[storage(write)]
    fn set_s_dot_t_dot_x(a: u64);
    #[storage(write)]
    fn set_s_dot_t_dot_y(b: u64);
    #[storage(write)]
    fn set_s_dot_t_dot_z(c: b256);
    #[storage(write)]
    fn set_s_dot_t_dot_boolean(boolean: bool);
    #[storage(write)]
    fn set_s_dot_t_dot_int8(int8: u8);
    #[storage(write)]
    fn set_s_dot_t_dot_int16(int16: u16);
    #[storage(write)]
    fn set_s_dot_t_dot_int32(int32: u32);
    #[storage(write)]
    fn set_e(e: E);
    #[storage(write)]
    fn set_string(s: str[40]);

    // Getters
    #[storage(read)]
    fn get_x() -> u64;
    #[storage(read)]
    fn get_y() -> b256;
    #[storage(read)]
    fn get_s() -> S;
    #[storage(read)]
    fn get_boolean() -> bool;
    #[storage(read)]
    fn get_int8() -> u8;
    #[storage(read)]
    fn get_int16() -> u16;
    #[storage(read)]
    fn get_int32() -> u32;
    #[storage(read)]
    fn get_s_dot_x() -> u64;
    #[storage(read)]
    fn get_s_dot_y() -> u64;
    #[storage(read)]
    fn get_s_dot_z() -> b256;
    #[storage(read)]
    fn get_s_dot_t() -> T;
    #[storage(read)]
    fn get_s_dot_t_dot_x() -> u64;
    #[storage(read)]
    fn get_s_dot_t_dot_y() -> u64;
    #[storage(read)]
    fn get_s_dot_t_dot_z() -> b256;
    #[storage(read)]
    fn get_s_dot_t_dot_boolean() -> bool;
    #[storage(read)]
    fn get_s_dot_t_dot_int8() -> u8;
    #[storage(read)]
    fn get_s_dot_t_dot_int16() -> u16;
    #[storage(read)]
    fn get_s_dot_t_dot_int32() -> u32;
    #[storage(read)]
    fn get_e() -> E;
    #[storage(read)]
    fn get_e2() -> E;
    #[storage(read)]
    fn get_string() -> str[40];

    // Operations
    #[storage(read, write)]
    fn add_to_s_dot_t_dot_x(k: u64);
    #[storage(read, write)]
    fn subtract_from_s_dot_t_dot_x(k: u64);
    #[storage(read, write)]
    fn multiply_by_s_dot_t_dot_x(k: u64);
    #[storage(read, write)]
    fn divide_s_dot_t_dot_x(k: u64);
    #[storage(read, write)]
    fn shift_left_s_dot_t_dot_x(k: u64);
    #[storage(read, write)]
    fn shift_right_s_dot_t_dot_x(k: u64);
}
