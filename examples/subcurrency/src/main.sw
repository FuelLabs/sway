// ANCHOR: body
contract;

use std::{address::Address, assert::assert, chain::auth::{AuthError, Sender, msg_sender}, hash::*, panic::panic, result::*, storage::{get, store}};

////////////////////////////////////////
// Event declarations
////////////////////////////////////////

// Events allow clients to react to changes in the contract.
// Unlike Solidity, events are simply structs.
// Note: Serialization is not yet implemented, therefore logging
//  of arbitrary structures will not work without manual
//  serialization.

/// Emitted when a token is sent.
struct Sent {
    from: Address,
    to: Address,
    amount: u64,
}

////////////////////////////////////////
// ABI declarations
////////////////////////////////////////

/// ABI definition for a subcurrency.
abi Token {
    // Mint new tokens and send to an address.
    // Can only be called by the contract creator.
    fn mint(receiver: Address, amount: u64);

    // Sends an amount of an existing token.
    // Can be called from any address.
    fn send(receiver: Address, amount: u64);
}

////////////////////////////////////////
// Constants
////////////////////////////////////////

/// Address of contract creator.
const MINTER: b256 = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;

////////////////////////////////////////
// Contract storage
////////////////////////////////////////

// Contract storage persists across transactions.
// Note: Contract storage mappings are not implemented yet.
const STORAGE_BALANCES: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

////////////////////////////////////////
// ABI definitions
////////////////////////////////////////

/// Contract implements the `Token` ABI.
impl Token for Contract {
    fn mint(receiver: Address, amount: u64) {
        let sender: Result<Sender, AuthError> = msg_sender();
        let sender = if let Sender::Address(addr) = sender.unwrap() {
            assert(addr.into() == MINTER);
        } else {
            panic(0);
        };

        // Increase the balance of receiver
        let storage_slot = hash_pair(STORAGE_BALANCES, receiver.into(), HashMethod::Sha256);
        let mut receiver_amount = get::<u64>(storage_slot);
        store(storage_slot, receiver_amount + amount);
    }

    fn send(receiver: Address, amount: u64) {
        let sender: Result<Sender, AuthError> = msg_sender();
        let sender = if let Sender::Address(addr) = sender.unwrap() {
            addr
        } else {
            panic(0);
        };

        // Reduce the balance of sender
        let sender_storage_slot = hash_pair(STORAGE_BALANCES, sender.into(), HashMethod::Sha256);
        let mut sender_amount = get::<u64>(sender_storage_slot);
        assert(sender_amount > amount);
        store(sender_storage_slot, sender_amount - amount);

        // Increase the balance of receiver
        let receiver_storage_slot = hash_pair(STORAGE_BALANCES, receiver.into(), HashMethod::Sha256);
        let mut receiver_amount = get::<u64>(receiver_storage_slot);
        store(receiver_storage_slot, receiver_amount + amount);
    }
}
// ANCHOR_END: body
