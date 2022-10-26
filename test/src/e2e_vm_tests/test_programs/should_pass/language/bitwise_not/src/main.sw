script;

fn main() -> bool {
    assert(!2u8 == 253u8);
    assert(!2u16 == 65533u16);
    assert(!2u32 == 4294967293u32);
    assert(!2u64 == 18446744073709551613u64);

    true
}
