// ANCHOR: body
contract;

use std::chain::*;
use std::hash::*;
use std::storage::*;

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
    fn send(sender: Address, receiver: Address, amount: u64);
}

////////////////////////////////////////
// Constants
////////////////////////////////////////

/// Address of contract creator.
const MINTER: Address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;

////////////////////////////////////////
// Contract storage
////////////////////////////////////////

// Contract storage persists across transactions.
// Note: Contract storage variables are not implemented yet.

const STORAGE_BALANCES: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

////////////////////////////////////////
// ABI definitions
////////////////////////////////////////

/// Contract implements the `Token` ABI.
impl Token for Contract {
    fn mint(receiver: Address, amount: u64) {
        // Note: authentication is not yet implemented, for now just trust params
        // See https://github.com/FuelLabs/sway/issues/195
        if receiver == MINTER {
            let storage_slot = hash_pair(STORAGE_BALANCES, MINTER, HashMethod::Sha256);

            let mut receiver_amount = get::<u64>(storage_slot);
            receiver_amount = receiver_amount + amount;
            store(storage_slot, receiver_amount);
        } else {
            // Revert with error `69`, chosen arbitrarily
            panic(69);
        }
    }

    fn send(sender: Address, receiver: Address, amount: u64) {
        let sender_storage_slot = hash_pair(STORAGE_BALANCES, sender, HashMethod::Sha256);

        let mut sender_amount = get::<u64>(sender_storage_slot);
        sender_amount = sender_amount - amount;
        store(sender_storage_slot, sender_amount);

        let receiver_storage_slot = hash_pair(STORAGE_BALANCES, receiver, HashMethod::Sha256);

        let mut receiver_amount = get::<u64>(receiver_storage_slot);
        receiver_amount = receiver_amount + amount;
        store(receiver_storage_slot, receiver_amount);
    }
}
// ANCHOR_END: body
