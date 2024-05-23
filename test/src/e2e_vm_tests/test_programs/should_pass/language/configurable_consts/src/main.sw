script;

configurable {
    BOOL: bool = true,
    U8: u8 = 2,
    U16: u16 = 2,
}

fn main() {
    assert(BOOL == true);
    assert(U8 == 2);
    assert(U16 == 2);
}
