# Subcurrency

```sway
contract;

use std::hash::*;
use std::storage::*;

////////////////////////////////////////
// Event declarations
////////////////////////////////////////

// Events allow clients to react to changes in the contract.
// Unlike Solidity, events are simply structs.

/// Emitted when a token is sent.
struct Sent {
    from: b256,
    to: b256,
    amount: u64,
}

////////////////////////////////////////
// ABI method parameter declarations
////////////////////////////////////////

/// Parameters for `mint` method.
struct ParamsMint {
    receiver: b256,
    amount: u64,
}

/// Parameters for `send` method.
struct ParamsSend {
    sender: b256,
    receiver: b256,
    amount: u64,
}

////////////////////////////////////////
// ABI declarations
////////////////////////////////////////

/// ABI definition for a subcurrency.
abi Token {
    // Mint new tokens and send to an address.
    // Can only be called by the contract creator.
    fn mint(gas: u64, coins: u64, color: b256, args: ParamsMint);

    // Sends an amount of an existing token.
    // Can be called from any address.
    fn send(gas: u64, coins: u64, color: b256, args: ParamsSend);
}

// Note: ABI methods for now must explicitly have as parameters:
//  gas to forward: u64
//  coins to forward: u64,
//  color of coins: b256

////////////////////////////////////////
// Constants
////////////////////////////////////////

/// Address of contract creator.
const minter: b256 = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;

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
    fn mint(gas: u64, coins: u64, color: b256, args: ParamsMint) {
        // Note: authentication is not yet implemented, for now just trust params
        if args.receiver == minter {
            let storage_slot = hash_pair(STORAGE_BALANCES, minter, HashMethod::Sha256);

            let mut amount = get_u64(storage_slot);
            amount = amount + args.amount;
            store_u64(storage_slot, amount);
        } else {
            // Note: Revert is not yet implemented
        }

    }

    fn send(gas: u64, coins: u64, color: b256, args: ParamsSend) {
        let sender_storage_slot = hash_pair(STORAGE_BALANCES, args.sender, HashMethod::Sha256);

        let mut sender_amount = get_u64(sender_storage_slot);
        sender_amount = sender_amount - args.amount;
        store_u64(sender_storage_slot, sender_amount);

        let receiver_storage_slot = hash_pair(STORAGE_BALANCES, args.receiver, HashMethod::Sha256);

        let mut receiver_amount = get_u64(receiver_storage_slot);
        receiver_amount = receiver_amount + args.amount;
        store_u64(receiver_storage_slot, receiver_amount);
    }
}
```
