contract;

use storage_access_abi::{S, StorageAccess, T};
use std::constants::NATIVE_ASSET_ID;

storage {
    x: u64, y: b256, s: S
}

impl StorageAccess for Contract {
    // Setters
    impure fn set_x(x: u64) {
        storage.x = x;
    }
    impure fn set_y(y: b256) {
        storage.y = y;
    }
    impure fn set_s(s: S) {
        storage.s = s;
    }
    impure fn set_s_dot_x(x: u64) {
        storage.s.x = x;
    }
    impure fn set_s_dot_y(y: u64) {
        storage.s.y = y;
    }
    impure fn set_s_dot_z(z: b256) {
        storage.s.z = z;
    }
    impure fn set_s_dot_t(t: T) {
        storage.s.t = t;
    }
    impure fn set_s_dot_t_dot_x(x: u64) {
        storage.s.t.x = x;
    }
    impure fn set_s_dot_t_dot_y(y: u64) {
        storage.s.t.y = y;
    }
    impure fn set_s_dot_t_dot_z(z: b256) {
        storage.s.t.z = z;
    }

    // Getters
    impure fn get_x() -> u64 {
        storage.x
    }
    impure fn get_y() -> b256 {
        storage.y
    }
    impure fn get_s() -> S {
        storage.s
    }
    impure fn get_s_dot_x() -> u64 {
        storage.s.x
    }
    impure fn get_s_dot_y() -> u64 {
        storage.s.y
    }
    impure fn get_s_dot_z() -> b256 {
        storage.s.z
    }
    impure fn get_s_dot_t() -> T {
        storage.s.t
    }
    impure fn get_s_dot_t_dot_x() -> u64 {
        storage.s.t.x
    }
    impure fn get_s_dot_t_dot_y() -> u64 {
        storage.s.t.y
    }
    impure fn get_s_dot_t_dot_z() -> b256 {
        storage.s.t.z
    }
}
