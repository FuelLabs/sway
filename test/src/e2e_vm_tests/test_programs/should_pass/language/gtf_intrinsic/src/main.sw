script;

const SOME_TX_FIELD = 0x42;
const SOME_OTHER_TX_FIELD = 0x77;

fn main() -> u64 {
    let u64_field = __gtf::<u64>(1, SOME_TX_FIELD);
    let b256_field = __gtf::<b256>(2, SOME_OTHER_TX_FIELD);
    0
}
