library context;
//! Functionality for accessing context-specific information about the current contract or message.

use ::contract_id::ContractId;

/// Get the current contract's id when called in an internal context.
/// **Note !** If called in an external context, this will **not** return a contract ID.
// @dev If called externally, will actually return a pointer to the transaction ID.
pub fn contract_id() -> b256 {
    asm() {
        fp: b256
    }
}

/// Get the current contracts balance of token `token_id`
pub fn this_balance(token_id: b256) -> u64 {
    asm(balance, token: token_id) {
        bal balance token fp;
        balance: u64
    }
}

/// Get the balance of token `token_id` for any contract `contract_id`
pub fn balance_of_contract(asset_id: b256, contract_id: ContractId) -> u64 {
    asm(balance, token: asset_id, contract: contract_id.value) {
        bal balance token contract;
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
