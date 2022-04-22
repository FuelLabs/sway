library panic;

/// Context-dependent:
/// will panic if used in a predicate
/// will revert if used in a contract
pub fn panic(code: u64) {
    asm(r1: code) {
        rvrt r1;
    }
}
