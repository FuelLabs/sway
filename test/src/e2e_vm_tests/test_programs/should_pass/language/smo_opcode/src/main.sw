script;

// Use local constant to avoid importing `std`.
const ZERO_B256: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

fn main() -> bool {
    asm(recipient: ZERO_B256, msg_len: 0, output: 0, coins: 0) {
        smo recipient msg_len coins output;
    }
    true
}
