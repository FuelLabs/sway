library token;
//! Functionality for performing common operations on tokens.

/// Mint `amount` coins of the current contract's `asset_id`.
pub fn mint(amount: u64) {
    asm(r1: amount) {
        mint r1;
    }
}

/// Burn `amount` coins of the current contract's `asset_id`.
pub fn burn(amount: u64) {
    asm(r1: amount) {
        burn r1;
    }
}
