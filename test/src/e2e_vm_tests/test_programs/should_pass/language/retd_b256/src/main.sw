script;

// a b256 is bigger than a word, so RETD should be used instead of RET.
fn main() -> b256 {
    let a = 0x0000000000000000000000000000000000000000000000000000000000000000;
    asm(r1: a, r2: 0x0000000000000000000000000000000000000000000000000000000000000000) {
        log r1 r2 zero zero;
        zero
    };
    return a;
}
