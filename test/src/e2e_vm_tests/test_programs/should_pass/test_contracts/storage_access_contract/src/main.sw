contract;

use storage_access_abi::{S, StorageAccess, T};

storage {
    x: u64,
    y: b256,
    s: S,
    boolean: bool,
    int8: u8,
    int16: u16,
    int32: u32,
}

impl StorageAccess for Contract {
    // Setters
    fn set_x(x: u64) {
        storage.x = x;
    }
    fn set_y(y: b256) {
        storage.y = y;
    }
    fn set_s(s: S) {
        storage.s = s;
    }
    fn set_boolean(boolean: bool) {
        storage.boolean = boolean;
    }
    fn set_int8(int8: u8) {
        storage.int8 = int8;
    }
    fn set_int16(int16: u16) {
        storage.int16 = int16;
    }
    fn set_int32(int32: u32) {
        storage.int32 = int32;
    }
    fn set_s_dot_x(x: u64) {
        storage.s.x = x;
    }
    fn set_s_dot_y(y: u64) {
        storage.s.y = y;
    }
    fn set_s_dot_z(z: b256) {
        storage.s.z = z;
    }
    fn set_s_dot_t(t: T) {
        storage.s.t = t;
    }
    fn set_s_dot_t_dot_x(x: u64) {
        storage.s.t.x = x;
    }
    fn set_s_dot_t_dot_y(y: u64) {
        storage.s.t.y = y;
    }
    fn set_s_dot_t_dot_z(z: b256) {
        storage.s.t.z = z;
    }
    fn set_s_dot_t_dot_boolean(boolean: bool) {
        storage.s.t.boolean = boolean;
    }
    fn set_s_dot_t_dot_int8(int8: u8) {
        storage.s.t.int8 = int8;
    }
    fn set_s_dot_t_dot_int16(int16: u16) {
        storage.s.t.int16 = int16;
    }
    fn set_s_dot_t_dot_int32(int32: u32) {
        storage.s.t.int32 = int32;
    }

    // Getters
    fn get_x() -> u64 {
        storage.x
    }
    fn get_y() -> b256 {
        storage.y
    }
    fn get_s() -> S {
        storage.s
    }
    fn get_boolean() -> bool {
        storage.boolean
    }
    fn get_int8() -> u8 {
        storage.int8
    }
    fn get_int16() -> u16 {
        storage.int16
    }
    fn get_int32() -> u32 {
        storage.int32
    }
    fn get_s_dot_x() -> u64 {
        storage.s.x
    }
    fn get_s_dot_y() -> u64 {
        storage.s.y
    }
    fn get_s_dot_z() -> b256 {
        storage.s.z
    }
    fn get_s_dot_t() -> T {
        storage.s.t
    }
    fn get_s_dot_t_dot_x() -> u64 {
        storage.s.t.x
    }
    fn get_s_dot_t_dot_y() -> u64 {
        storage.s.t.y
    }
    fn get_s_dot_t_dot_z() -> b256 {
        storage.s.t.z
    }
    fn get_s_dot_t_dot_boolean() -> bool {
        storage.s.t.boolean
    }
    fn get_s_dot_t_dot_int8() -> u8 {
        storage.s.t.int8
    }
    fn get_s_dot_t_dot_int16() -> u16 {
        storage.s.t.int16
    }
    fn get_s_dot_t_dot_int32() -> u32 {
        storage.s.t.int32
    }
}
