script;
use storage_access_abi::{S, StorageAccess, T};
use std::assert::assert;

fn main() -> bool {
    let contract_id = 0x1c305447488d8da2c5bd6b025222970b699edd84aa405f15cace2b33a13ddaf4;
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
        },
    };
    caller.set_s(s);
    assert(caller.get_s_dot_x() == 3);
    assert(caller.get_s_dot_y() == 4);
    assert(caller.get_s_dot_z() == 0x0000000000000000000000000000000000000000000000000000000000000002);
    assert(caller.get_s_dot_t_dot_x() == 5);
    assert(caller.get_s_dot_t_dot_y() == 6);
    assert(caller.get_s_dot_t_dot_z() == 0x0000000000000000000000000000000000000000000000000000000000000003);

    // Test 4
    let t = T {
        x: 7,
        y: 8,
        z: 0x0000000000000000000000000000000000000000000000000000000000000004,
    };
    caller.set_s_dot_t(t);
    assert(caller.get_s_dot_t_dot_x() == 7);
    assert(caller.get_s_dot_t_dot_y() == 8);
    assert(caller.get_s_dot_t_dot_z() == 0x0000000000000000000000000000000000000000000000000000000000000004);

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
