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
    string: str[40] = __to_str_array("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
}

impl StorageAccess for Contract {
    // Setters
    #[storage(write)]fn set_x(x: u64) {
        storage.x.write(x);
    }
    #[storage(write)]fn set_y(y: b256) {
        storage.y.write(y);
    }
    #[storage(write)]fn set_s(s: S) {
        storage.s.write(s);
    }
    #[storage(write)]fn set_boolean(boolean: bool) {
        log(boolean);
        storage.boolean.write(boolean);
    }
    #[storage(write)]fn set_int8(int8: u8) {
        storage.int8.write(int8);
    }
    #[storage(write)]fn set_int16(int16: u16) {
        storage.int16.write(int16);
    }
    #[storage(write)]fn set_int32(int32: u32) {
        storage.int32.write(int32);
    }
    #[storage(write)]fn set_s_dot_x(x: u64) {
        storage.s.x.write(x);
    }
    #[storage(write)]fn set_s_dot_y(y: u64) {
        storage.s.y.write(y);
    }
    #[storage(write)]fn set_s_dot_z(z: b256) {
        storage.s.z.write(z);
    }
    #[storage(write)]fn set_s_dot_t(t: T) {
        storage.s.t.write(t);
    }
    #[storage(write)]fn set_s_dot_t_dot_x(x: u64) {
        storage.s.t.x.write(x);
    }
    #[storage(write)]fn set_s_dot_t_dot_y(y: u64) {
        storage.s.t.y.write(y);
    }
    #[storage(write)]fn set_s_dot_t_dot_z(z: b256) {
        storage.s.t.z.write(z);
    }
    #[storage(write)]fn set_s_dot_t_dot_boolean(boolean: bool) {
        storage.s.t.boolean.write(boolean);
    }
    #[storage(write)]fn set_s_dot_t_dot_int8(int8: u8) {
        storage.s.t.int8.write(int8);
    }
    #[storage(write)]fn set_s_dot_t_dot_int16(int16: u16) {
        storage.s.t.int16.write(int16);
    }
    #[storage(write)]fn set_s_dot_t_dot_int32(int32: u32) {
        storage.s.t.int32.write(int32);
    }
    #[storage(write)]fn set_e(e: E) {
        storage.e.write(e);
    }
    #[storage(write)]fn set_string(string: str[40]) {
        storage.string.write(string);
    }

    // Getters
    #[storage(read)]fn get_x() -> u64 {
        storage.x.read()
    }
    #[storage(read)]fn get_y() -> b256 {
        storage.y.read()
    }
    #[storage(read)]fn get_s() -> S {
        storage.s.read()
    }
    #[storage(read)]fn get_boolean() -> bool {
        storage.boolean.read()
    }
    #[storage(read)]fn get_int8() -> u8 {
        storage.int8.read()
    }
    #[storage(read)]fn get_int16() -> u16 {
        storage.int16.read()
    }
    #[storage(read)]fn get_int32() -> u32 {
        storage.int32.read()
    }
    #[storage(read)]fn get_s_dot_x() -> u64 {
        storage.s.x.read()
    }
    #[storage(read)]fn get_s_dot_y() -> u64 {
        storage.s.y.read()
    }
    #[storage(read)]fn get_s_dot_z() -> b256 {
        storage.s.z.read()
    }
    #[storage(read)]fn get_s_dot_t() -> T {
        storage.s.t.read()
    }
    #[storage(read)]fn get_s_dot_t_dot_x() -> u64 {
        storage.s.t.x.read()
    }
    #[storage(read)]fn get_s_dot_t_dot_y() -> u64 {
        storage.s.t.y.read()
    }
    #[storage(read)]fn get_s_dot_t_dot_z() -> b256 {
        storage.s.t.z.read()
    }
    #[storage(read)]fn get_s_dot_t_dot_boolean() -> bool {
        storage.s.t.boolean.read()
    }
    #[storage(read)]fn get_s_dot_t_dot_int8() -> u8 {
        storage.s.t.int8.read()
    }
    #[storage(read)]fn get_s_dot_t_dot_int16() -> u16 {
        storage.s.t.int16.read()
    }
    #[storage(read)]fn get_s_dot_t_dot_int32() -> u32 {
        storage.s.t.int32.read()
    }
    #[storage(read)]fn get_e() -> E {
        storage.e.read()
    }
    #[storage(read)]fn get_e2() -> E {
        storage.e2.read()
    }
    #[storage(read)]fn get_string() -> str[40] {
        storage.string.read()
    }
}

#[test]
fn collect_storage_access_contract_gas_usages() {
    let caller = abi(StorageAccess, CONTRACT_ID);
    let _ = caller.set_x(0);
    let _ = caller.set_y(b256::zero());
    let _ = caller.set_s(S {
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
    });
    let _ = caller.set_boolean(false);
    let _ = caller.set_int8(0);
    let _ = caller.set_int16(0);
    let _ = caller.set_int32(0);
    let _ = caller.set_s_dot_x(0);
    let _ = caller.set_s_dot_y(0);
    let _ = caller.set_s_dot_z(b256::zero());
    let _ = caller.set_s_dot_t(T {
        x: 1,
        y: 2,
        z: 0x0000000000000000000000000000000000000000000000000000000000000003,
        boolean: true,
        int8: 4,
        int16: 5,
        int32: 6,
    },);
    let _ = caller.set_s_dot_t_dot_x(0);
    let _ = caller.set_s_dot_t_dot_y(0);
    let _ = caller.set_s_dot_t_dot_z(b256::zero());
    let _ = caller.set_s_dot_t_dot_boolean(false);
    let _ = caller.set_s_dot_t_dot_int8(0);
    let _ = caller.set_s_dot_t_dot_int16(0);
    let _ = caller.set_s_dot_t_dot_int32(0);
    let _ = caller.set_e(E::A(0));
    let _ = caller.set_string(__to_str_array("BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"));

    let _ = caller.get_x();
    let _ = caller.get_y();
    let _ = caller.get_s();
    let _ = caller.get_boolean();
    let _ = caller.get_int8();
    let _ = caller.get_int16();
    let _ = caller.get_int32();
    let _ = caller.get_s_dot_x();
    let _ = caller.get_s_dot_y();
    let _ = caller.get_s_dot_z();
    let _ = caller.get_s_dot_t();
    let _ = caller.get_s_dot_t_dot_x();
    let _ = caller.get_s_dot_t_dot_y();
    let _ = caller.get_s_dot_t_dot_z();
    let _ = caller.get_s_dot_t_dot_boolean();
    let _ = caller.get_s_dot_t_dot_int8();
    let _ = caller.get_s_dot_t_dot_int16();
    let _ = caller.get_s_dot_t_dot_int32();
    let _ = caller.get_e();
    let _ = caller.get_e2();
    let _ = caller.get_string();
}
