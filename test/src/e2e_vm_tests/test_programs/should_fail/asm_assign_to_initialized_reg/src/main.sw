script;

fn main() -> u64 {
    asm(r1: 0, r2: 0, r3, r4) {
        move r1 r2;
        move r2 r1;
        move r3 r1;
        move r4 r2;
    };
    0
}
