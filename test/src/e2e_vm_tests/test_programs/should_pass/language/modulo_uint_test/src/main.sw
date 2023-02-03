script;

fn main() -> bool {
    let uint64_test1: u64 = 100000000000;
    let uint32_test1: u32 = 1000000000;
    let uint16_test1: u16 = 10000;
    let uint8_test1: u8 = 100;

    // Ensure 0 remainder returns correctly
    assert(uint64_test1 % 100u64 == 0);
    assert(uint32_test1 % 100u32 == 0);
    assert(uint16_test1 % 100u16 == 0);
    assert(uint8_test1 % 100u8 == 0);

    let uint64_test2: u64 = 100000000005;
    let uint32_test2: u32 = 1000000005;
    let uint16_test2: u16 = 10005;
    let uint8_test2: u8 = 105;

    // Ensure non zero remainder returns correctly
    assert(uint64_test2 % 100u64 == 5);
    assert(uint32_test2 % 100u32 == 5);
    assert(uint16_test2 % 100u16 == 5);
    assert(uint8_test2 % 100u8 == 5);

    true
}
