library context;

/// get the contract id for the current contract
pub fn this_id() -> b256 {
    asm() {
            fp: b256
        }
}

/// get the value of coins being sent
pub fn msg_value() -> u64 {
    asm() {
        bal: u64
    }
}

/// get the token_id (color) of coins being sent
pub fn msg_color() -> b256 {
    asm(token_id) {
        addi token_id fp i32;
        token_id: b256
    }
}

// get the remaining gas in the context
pub fn msg_gas() -> u64 {
    asm() {
        cgas: u64
    }
}

// get the remaining gas globally
pub fn global_gas() -> u64 {
    ggas: u64
}
