library revert;

/// Context-dependent:
/// will revert if used in a predicate
/// will revert if used in a contract
pub fn revert(code: u64) {
    asm(r1: code) {
        rvrt r1;
    }
}
