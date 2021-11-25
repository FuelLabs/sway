library context;
//! Functionality for accessing context-specific information about the current contract or message.

/// Get the current contract's id when called in an internal context.
/// **Note !** If called in an external context, this will **not** return a contract ID.
pub fn this_id() -> b256 {
    asm() {
        fp: b256
    }
}

/// Get the value of coins being sent.
pub fn msg_value() -> u64 {
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
