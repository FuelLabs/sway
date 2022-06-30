contract;

use storage_access_abi::{E, S, StorageAccess, T};

storage {
    x: u64 = 64,
    y: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101,
    s: S = S {
        x: 1,
        y: 2,
        z: 0x0000000000000000000000000000000000000000000000000000000000000003,
        t: T {
            x: 4,
            y: 5,
            z: 0x0000000000000000000000000000000000000000000000000000000000000006,
            boolean: true,
            int8: 7,
            int16: 8,
            int32: 9,
        },
    },
    boolean: bool = true,
    int8: u8 = 8,
    int16: u16 = 16,
    int32: u32 = 32,
    e: E = E::B(T {
        x: 1,
        y: 2,
        z: 0x0000000000000000000000000000000000000000000000000000000000000003,
        boolean: true,
        int8: 4,
        int16: 5,
        int32: 6,
    },
    ), e2: E = E::A(777),
    string: str[40] = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
}

impl StorageAccess for Contract {
    // Setters
    #[storage(write)]fn set_x(x: u64) {
        storage.x = x;
    }
    #[storage(write)]fn set_y(y: b256) {
        storage.y = y;
    }
    #[storage(write)]fn set_s(s: S) {
        storage.s = s;
    }
    #[storage(write)]fn set_boolean(boolean: bool) {
        storage.boolean = boolean;
    }
    #[storage(write)]fn set_int8(int8: u8) {
        storage.int8 = int8;
    }
    #[storage(write)]fn set_int16(int16: u16) {
        storage.int16 = int16;
    }
    #[storage(write)]fn set_int32(int32: u32) {
        storage.int32 = int32;
    }
    #[storage(write)]fn set_s_dot_x(x: u64) {
        storage.s.x = x;
    }
    #[storage(write)]fn set_s_dot_y(y: u64) {
        storage.s.y = y;
    }
    #[storage(write)]fn set_s_dot_z(z: b256) {
        storage.s.z = z;
    }
    #[storage(write)]fn set_s_dot_t(t: T) {
        storage.s.t = t;
    }
    #[storage(write)]fn set_s_dot_t_dot_x(x: u64) {
        storage.s.t.x = x;
    }
    #[storage(write)]fn set_s_dot_t_dot_y(y: u64) {
        storage.s.t.y = y;
    }
    #[storage(write)]fn set_s_dot_t_dot_z(z: b256) {
        storage.s.t.z = z;
    }
    #[storage(write)]fn set_s_dot_t_dot_boolean(boolean: bool) {
        storage.s.t.boolean = boolean;
    }
    #[storage(write)]fn set_s_dot_t_dot_int8(int8: u8) {
        storage.s.t.int8 = int8;
    }
    #[storage(write)]fn set_s_dot_t_dot_int16(int16: u16) {
        storage.s.t.int16 = int16;
    }
    #[storage(write)]fn set_s_dot_t_dot_int32(int32: u32) {
        storage.s.t.int32 = int32;
    }
    #[storage(write)]fn set_e(e: E) {
        storage.e = e;
    }
    #[storage(write)]fn set_string(string: str[40]) {
        storage.string = string;
    }

    // Getters
    #[storage(read)]fn get_x() -> u64 {
        storage.x
    }
    #[storage(read)]fn get_y() -> b256 {
        storage.y
    }
    #[storage(read)]fn get_s() -> S {
        storage.s
    }
    #[storage(read)]fn get_boolean() -> bool {
        storage.boolean
    }
    #[storage(read)]fn get_int8() -> u8 {
        storage.int8
    }
    #[storage(read)]fn get_int16() -> u16 {
        storage.int16
    }
    #[storage(read)]fn get_int32() -> u32 {
        storage.int32
    }
    #[storage(read)]fn get_s_dot_x() -> u64 {
        storage.s.x
    }
    #[storage(read)]fn get_s_dot_y() -> u64 {
        storage.s.y
    }
    #[storage(read)]fn get_s_dot_z() -> b256 {
        storage.s.z
    }
    #[storage(read)]fn get_s_dot_t() -> T {
        storage.s.t
    }
    #[storage(read)]fn get_s_dot_t_dot_x() -> u64 {
        storage.s.t.x
    }
    #[storage(read)]fn get_s_dot_t_dot_y() -> u64 {
        storage.s.t.y
    }
    #[storage(read)]fn get_s_dot_t_dot_z() -> b256 {
        storage.s.t.z
    }
    #[storage(read)]fn get_s_dot_t_dot_boolean() -> bool {
        storage.s.t.boolean
    }
    #[storage(read)]fn get_s_dot_t_dot_int8() -> u8 {
        storage.s.t.int8
    }
    #[storage(read)]fn get_s_dot_t_dot_int16() -> u16 {
        storage.s.t.int16
    }
    #[storage(read)]fn get_s_dot_t_dot_int32() -> u32 {
        storage.s.t.int32
    }
    #[storage(read)]fn get_e() -> E {
        storage.e
    }
    #[storage(read)]fn get_e2() -> E {
        storage.e2
    }
    #[storage(read)]fn get_string() -> str[40] {
        storage.string
    }

    // Operations
    #[storage(read, write)]fn add_to_s_dot_t_dot_x(k: u64) {
        storage.s.t.x += k;
    }
    #[storage(read, write)]fn subtract_from_s_dot_t_dot_x(k: u64) {
        storage.s.t.x -= k;
    }
    #[storage(read, write)]fn multiply_by_s_dot_t_dot_x(k: u64) {
        storage.s.t.x *= k;
    }
    #[storage(read, write)]fn divide_s_dot_t_dot_x(k: u64) {
        storage.s.t.x /= k;
    }
    #[storage(read, write)]fn shift_left_s_dot_t_dot_x(k: u64) {
        storage.s.t.x <<= k;
    }
    #[storage(read, write)]fn shift_right_s_dot_t_dot_x(k: u64) {
        storage.s.t.x >>= k;
    }
}
