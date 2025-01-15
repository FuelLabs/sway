script;

pub fn main() -> bool {
    true
}

#[test]
fn side_effect_register_not_cleared() -> u64 {
    let a = asm(a, b) {
        movi b i16;     // b = 16
        aloc b;         // buf1 = [0;16]
        movi b i0;      // b = 0
        sw hp b i0;     // buf1[0:8] = b = 0
        movi a i0;      // a = 0
        add a hp a;     // a = &buf1
        movi b i16;     // b = 16
        aloc b;         // buf2 = [0;16]
        movi b i1;      // b = 1
        sw hp b i0;     // buf2[0:8] = b = 1
        lw a a i0;      // expected : a = buf1[0:8] = 0         real : a = buf2[0:8] = 1
        a
    };
    assert(a == 0);
    a
}
