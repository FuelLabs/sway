script;

fn main() {
    asm(r1: 1u64, r2: 2u32, r3: 3u32, r4: 4u32) {
        ecal r1 r2 r3 r4;
    }
}
