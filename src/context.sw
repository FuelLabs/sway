library context;
//! Functionality for accessing context-specific information about the current contract or message.

use ::contract_id::ContractId;

/// Get the current contract's id when called in an internal context.
/// **Note !** If called in an external context, this will **not** return a contract ID.
// @dev If called externally, will actually return a pointer to the transaction ID.
pub fn contract_id() -> ContractId {
    ~ContractId::from(asm() {
        fp: b256
    })
}

/// Get the current contracts balance of coin `asset_id`
pub fn this_balance(asset_id: b256) -> u64 {
    let this_id = contract_id();
    asm(balance, token: asset_id, contract_id: this_id) {
        bal balance token contract_id;
        balance: u64
    }
}

/// Get the balance of coin `asset_id` for any contract `contract_id`
pub fn balance_of_contract(asset_id: b256, contract_id: ContractId) -> u64 {
    asm(balance, token: asset_id, contract: contract_id.value) {
        bal balance token contract;
        balance: u64
    }
}

/// Get the amount of units of `msg_asset_id()` being sent.
pub fn msg_amount() -> u64 {
    asm() {
        bal: u64
    }
}

/// Get the asset_id of coins being sent.
pub fn msg_asset_id() -> ContractId {
    ~ContractId::from(asm(asset_id) {
        addi asset_id fp i32;
        asset_id: b256
    })
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
