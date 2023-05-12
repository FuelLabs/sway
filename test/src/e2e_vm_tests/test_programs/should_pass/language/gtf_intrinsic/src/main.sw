script;

const SOME_TX_FIELD = 0x42;
const SOME_OTHER_TX_FIELD = 0x77;

fn main() -> u64 {
    // Test expected to compile but revert because `fuel-core` does not support `gtf` yet.
    let _u64_field = __gtf::<u64>(1, SOME_TX_FIELD);
    let _b256_field = __gtf::<b256>(2, SOME_OTHER_TX_FIELD);
    0
}
