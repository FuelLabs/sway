// ANCHOR: body
contract;

use std::{
    address::Address,
    assert::assert,
    chain::auth::{AuthError, msg_sender},
    hash::sha256,
    identity::Identity,
    logging::log,
    result::*,
    revert::revert,
    storage::{get, store}
};

////////////////////////////////////////
// Event declarations
////////////////////////////////////////

// Events allow clients to react to changes in the contract.
// Unlike Solidity, events are simply structs.
// Note: Logging of arbitrary stack types is supported, however they cannot yet
//  be decoded on the SDK side.

/// Emitted when a token is sent.
struct Sent {
    from: Address,
    to: Address,
    amount: u64,
}

////////////////////////////////////////
// ABI method declarations
////////////////////////////////////////

/// ABI for a subcurrency.
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
// Note: Contract storage mappings are not implemented yet, so the domain
//  separator needs to be provided manually.
const STORAGE_BALANCES: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

////////////////////////////////////////
// ABI definitions
////////////////////////////////////////

/// Contract implements the `Token` ABI.
impl Token for Contract {
    fn mint(receiver: Address, amount: u64) {
        // Note: The return type of `msg_sender()` can be inferred by the
        // compiler. It is shown here for explicitness.
        let sender: Result<Identity, AuthError> = msg_sender();
        let sender: Address = match sender.unwrap() {
            Identity::Address(addr) => {
                assert(addr == ~Address::from(MINTER));
                addr
            },
            _ => {
                revert(0);
            },
        };

        // Increase the balance of receiver
        let storage_slot = sha256((STORAGE_BALANCES, receiver));
        let receiver_amount = get::<u64>(storage_slot);
        store(storage_slot, receiver_amount + amount);
    }

    fn send(receiver: Address, amount: u64) {
        // Note: The return type of `msg_sender()` can be inferred by the
        // compiler. It is shown here for explicitness.
        let sender: Result<Identity, AuthError> = msg_sender();
        let sender: Address = match sender.unwrap() {
            Identity::Address(addr) => {
                assert(addr == ~Address::from(MINTER));
                addr
            },
            _ => {
                revert(0);
            },
        };

        // Reduce the balance of sender
        let sender_storage_slot = sha256((STORAGE_BALANCES, sender));
        let sender_amount = get::<u64>(sender_storage_slot);
        assert(sender_amount > amount);
        store(sender_storage_slot, sender_amount - amount);

        // Increase the balance of receiver
        let receiver_storage_slot = sha256((STORAGE_BALANCES, receiver));
        let receiver_amount = get::<u64>(receiver_storage_slot);
        store(receiver_storage_slot, receiver_amount + amount);

        log(Sent {
            from: sender, to: receiver, amount: amount
        });
    }
}
// ANCHOR_END: body
