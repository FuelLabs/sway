contract;

use std::{address::Address, assert::assert, chain::auth::{AuthError, Sender, msg_sender}, constants::NATIVE_ASSET_ID, context::{call_frames::msg_asset_id, msg_amount}, contract_id::ContractId, panic::panic, result::*, token::transfer_to_output};

const OWNER_ADDRESS: b256 = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;

storage {
    balance: u64,
}

abi Wallet {
    fn receive_funds();
    fn send_funds(amount_to_send: u64, recipient_address: Address);
}

impl Wallet for Contract {
    fn receive_funds() {
        if (msg_asset_id()).into() == NATIVE_ASSET_ID {
            storage.balance = storage.balance + msg_amount();
        };
    }

    fn send_funds(amount_to_send: u64, recipient_address: Address) {
        let sender: Result<Sender, AuthError> = msg_sender();
        if let Sender::Address(addr) = sender.unwrap() {
            assert(addr.into() == OWNER_ADDRESS);
        } else {
            panic(0);
        };

        let current_balance = storage.balance;
        assert(current_balance > amount_to_send);
        storage.balance = current_balance - amount_to_send;
        transfer_to_output(amount_to_send, ~ContractId::from(NATIVE_ASSET_ID), recipient_address);
    }
}
