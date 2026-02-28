script;

fn main() -> u64 {
    asm(r1, r0: 0) {
        addi r1 r0 i3;
        log r0 r0 r0 r0; // PATCH: 0x50 010000 010000 000000 000100
        addi r1 r1 i5;
        r1: u64
    }
}
