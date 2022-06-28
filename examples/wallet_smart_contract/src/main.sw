contract;

use std::{
    address::Address,
    assert::assert,
    chain::auth::{AuthError, msg_sender},
    constants::BASE_ASSET_ID,
    context::{call_frames::msg_asset_id, msg_amount},
    contract_id::ContractId,
    identity::Identity,
    result::*,
    revert::revert,
    token::transfer_to_output,
};

const OWNER_ADDRESS: b256 = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;

storage {
    balance: u64,
}

abi Wallet {
    #[storage(read, write)]fn receive_funds();
    #[storage(read, write)]fn send_funds(amount_to_send: u64, recipient_address: Address);
}

impl Wallet for Contract {
    #[storage(read, write)]fn receive_funds() {
        if msg_asset_id() == ~ContractId::from(BASE_ASSET_ID) {
            // If we received `BASE_ASSET_ID` then keep track of the balance.
            // Otherwise, we're receiving other native assets and don't care
            // about our balance of tokens.
            storage.balance = storage.balance + msg_amount();
        }
    }

    #[storage(read, write)]fn send_funds(amount_to_send: u64, recipient_address: Address) {
        // Note: The return type of `msg_sender()` can be inferred by the
        // compiler. It is shown here for explicitness.
        let sender: Result<Identity, AuthError> = msg_sender();
        match sender.unwrap() {
            Identity::Address(addr) => {
                assert(addr == ~Address::from(OWNER_ADDRESS));
            },
            _ => {
                revert(0);
            },
        };

        let current_balance = storage.balance;
        assert(current_balance >= amount_to_send);

        storage.balance = current_balance - amount_to_send;
        // Note: `transfer_to_output()` is not a call and thus not an
        // interaction. Regardless, this code conforms to
        // checks-effects-interactions to avoid re-entrancy.
        transfer_to_output(amount_to_send, ~ContractId::from(BASE_ASSET_ID), recipient_address);
    }
}
