predicate;

fn main() -> bool {
    asm(r1: 0, r2: 0, r3: 0) {
        ldc r1 r2 r3 i0;
    }
    true
}