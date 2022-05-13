script;
use storage_access_abi::{S, StorageAccess, T};
use std::assert::assert;

fn main() -> bool {
    let contract_id = 0x6acd5f067ccfaab80d77065e3613a41523fc951acf73e24e51ffb64a9f83826a;
    let caller = abi(StorageAccess, contract_id);

    // Test 1
    caller.set_x(1);
    assert(caller.get_x() == 1);

    // Test 2
    caller.set_y(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(caller.get_y() == 0x0000000000000000000000000000000000000000000000000000000000000001);

    // Test 3
    let s = S {
        x: 3,
        y: 4,
        z: 0x0000000000000000000000000000000000000000000000000000000000000002,
        t: T {
            x: 5,
            y: 6,
            z: 0x0000000000000000000000000000000000000000000000000000000000000003,
            boolean: true,
            int8: 88,
            int16: 1616,
            int32: 3232,
        },
    };
    caller.set_s(s);
    assert(caller.get_s_dot_x() == 3);
    assert(caller.get_s_dot_y() == 4);
    assert(caller.get_s_dot_z() == 0x0000000000000000000000000000000000000000000000000000000000000002);
    assert(caller.get_s_dot_t_dot_x() == 5);
    assert(caller.get_s_dot_t_dot_y() == 6);
    assert(caller.get_s_dot_t_dot_z() == 0x0000000000000000000000000000000000000000000000000000000000000003);

    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    caller.set_boolean(false);
    assert(caller.get_boolean() == false);
    caller.set_int8(8u8);
    assert(caller.get_int8() == 8u8);
    caller.set_int16(16u16);
    assert(caller.get_int16() == 16u16);
    caller.set_int32(32u32);
    assert(caller.get_int32() == 32u32);

    // Test 4
    let t = T {
        x: 7,
        y: 8,
        z: 0x0000000000000000000000000000000000000000000000000000000000000004,
        boolean: true,
        int8: 222u8,
        int16: 1616u16,
        int32: 323232u32,
    };
    caller.set_s_dot_t(t);
    assert(caller.get_s_dot_t_dot_x() == 7);
    assert(caller.get_s_dot_t_dot_y() == 8);
    assert(caller.get_s_dot_t_dot_z() == 0x0000000000000000000000000000000000000000000000000000000000000004);
    assert(caller.get_s_dot_t_dot_boolean() == true);
    assert(caller.get_s_dot_t_dot_int8() == 222u8);
    assert(caller.get_s_dot_t_dot_int16() == 1616u16);
    assert(caller.get_s_dot_t_dot_int32() == 323232u32);

    // Test 5
    caller.set_s_dot_x(9);
    assert(caller.get_s_dot_x() == 9);

    // Test 6
    caller.set_s_dot_y(10);
    assert(caller.get_s_dot_y() == 10);

    // Test 7
    caller.set_s_dot_z(0x0000000000000000000000000000000000000000000000000000000000000005);
    assert(caller.get_s_dot_z() == 0x0000000000000000000000000000000000000000000000000000000000000005);

    // Test 8
    caller.set_s_dot_t_dot_x(11);
    assert(caller.get_s_dot_t_dot_x() == 11);

    // Test 9
    caller.set_s_dot_t_dot_y(12);
    assert(caller.get_s_dot_t_dot_y() == 12);

    // Test 10
    caller.set_s_dot_t_dot_z(0x0000000000000000000000000000000000000000000000000000000000000006);
    assert(caller.get_s_dot_t_dot_z() == 0x0000000000000000000000000000000000000000000000000000000000000006);

    caller.set_s_dot_t_dot_boolean(true);
    assert(caller.get_s_dot_t_dot_boolean() == true);

    caller.set_s_dot_t_dot_int8(22u8);
    assert(caller.get_s_dot_t_dot_int8() == 22u8);

    caller.set_s_dot_t_dot_int16(33u16);
    assert(caller.get_s_dot_t_dot_int16() == 33u16);

    caller.set_s_dot_t_dot_int32(44u32);
    assert(caller.get_s_dot_t_dot_int32() == 44u32);

    // Test 11
    let s = caller.get_s();
    assert(s.x == 9);
    assert(s.y == 10);
    assert(s.z == 0x0000000000000000000000000000000000000000000000000000000000000005);
    assert(s.t.x == 11);
    assert(s.t.y == 12);
    assert(s.t.z == 0x0000000000000000000000000000000000000000000000000000000000000006);

    // Test 12
    let t = caller.get_s_dot_t();
    assert(t.x == 11);
    assert(t.y == 12);
    assert(t.z == 0x0000000000000000000000000000000000000000000000000000000000000006);

    true
}
