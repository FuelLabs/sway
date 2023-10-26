script;

fn main() -> u64 {
    // Returning an unitialized register is not ok
    let _ = asm(r1) {
        r1: u64
    };

    // Reading unitialized register is not ok
    asm(r2) {
        sw r2 r2 i0;
    };

    // Writing before reading unitialized register is ok
    asm(r3) {
        movi r3 i0;
        sw r3 r3 i0;
    };

    // Writing before returning unitialized register is ok
    let _ = asm(r4) {
        movi r4 i0;
        r4: u64
    };

    // Shadowing a variable is a warning
    let r5 = 0;
    asm(r5) {};

    0
}
