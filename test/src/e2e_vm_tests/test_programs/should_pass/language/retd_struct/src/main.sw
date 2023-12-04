script;

struct BiggerThanAWord {
    field_1: u64,
    field_2: b256,
    field_3: [u8; 2],
}

fn main() -> BiggerThanAWord {
    BiggerThanAWord {
        field_1: 99999u64,
        field_2: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff,
        field_3: [1, 2]
    }
}
