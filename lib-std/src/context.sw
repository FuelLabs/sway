library context;
//! Functionality for accessing context-specific information about the current contract or message.

/// Get the current contract's id when called in an internal context.
/// **Note !** If called in an external context, this will **not** return a contract ID.
// @dev If called externally, will actually return a pointer to the transaction ID.
pub fn contract_id() -> b256 {
    asm() {
        fp: b256
    }
}

/// Get the current contracts balance of asset `id`
// @todo consider setting `contract_id` to ZERO to signify this contract (in which case the id is at $fp), otherwise use the contract_id arg
pub fn balance(asset_id: b256, contract_id: b256) -> u64 {
    asm(balance) {
        bal balance asset_id contract_id;
        balance: u64
    }
}

/// Get the amount of units of `msg_token_id()` being sent.
pub fn msg_amount() -> u64 {
    asm() {
        bal: u64
    }
}

/// Get the token_id of coins being sent.
pub fn msg_token_id() -> b256 {
    asm(token_id) {
        addi token_id fp i32;
        token_id: b256
    }
}

/// Get the remaining gas in the context.
pub fn gas() -> u64 {
    asm() {
        cgas: u64
    }
}

/// Get the remaining gas globally.
pub fn global_gas() -> u64 {
    asm() {
        ggas: u64
    }
}
