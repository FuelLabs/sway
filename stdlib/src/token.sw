library token;

use ::address:: Address;

// @todo add mint func
// @todo add burn func
// @todo add transfer_to_output func
// need transfer func?
// available spec opcodes
// MINT, BURN, TR, TRO
// @todo understand variable outputs

/// Mint `n` coins of the current contract's token_id.
pub fn mint(n: u64) {
    asm(r1: n) {
        mint r1;
    }
}

/// Burn `n` coins of the current contract's token_id.
pub fn burn(n: u64) {
    asm(r1: n) {
        burn r1;
    }
}

pub fn transfer_to_output(coins: u64, color: b256, recipient: Address) {
    // unimplemented
}

// pub fn balance() {}  // does it belong here?