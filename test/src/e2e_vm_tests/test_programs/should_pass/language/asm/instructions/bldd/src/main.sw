library;

// Intentionally not using `b256::zero()` to avoid dependency to `core`.
const B256_ZERO: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

pub fn main() {
    asm(r1: B256_ZERO, r2: B256_ZERO, r3: 42u64, r4: 21u64) {
        bldd r1 r2 r3 r4;
    }
}
