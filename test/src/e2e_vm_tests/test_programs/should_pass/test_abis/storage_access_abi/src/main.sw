library;

pub struct S {
    pub x: u64,
    pub y: u64,
    pub z: b256,
    pub t: T,
}

pub struct T {
    pub x: u64,
    pub y: u64,
    pub z: b256,
    pub boolean: bool,
    pub int8: u8,
    pub int16: u16,
    pub int32: u32,
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
}
