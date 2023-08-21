// ANCHOR: body
contract;

use std::hash::*;

////////////////////////////////////////
// Event declarations
////////////////////////////////////////
//
// Events allow clients to react to changes in the contract.
// Unlike Solidity, events are simply structs.
//
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
    #[storage(read, write)]
    fn mint(receiver: Address, amount: u64);

    // Sends an amount of an existing token.
    // Can be called from any address.
    #[storage(read, write)]
    fn send(receiver: Address, amount: u64);
}

////////////////////////////////////////
// Constants
////////////////////////////////////////
/// Address of contract creator.
const MINTER = Address::from(0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b);

////////////////////////////////////////
// Contract storage
////////////////////////////////////////
// Contract storage persists across transactions.
storage {
    balances: StorageMap<Address, u64> = StorageMap::<Address, u64> {},
}

////////////////////////////////////////
// ABI definitions
////////////////////////////////////////
/// Contract implements the `Token` ABI.
impl Token for Contract {
    #[storage(read, write)]
    fn mint(receiver: Address, amount: u64) {
        let sender = msg_sender().unwrap();
        let sender: Address = match sender {
            Identity::Address(addr) => {
                assert(addr == MINTER);
                addr
            },
            _ => revert(0),
        };

        // Increase the balance of receiver
        storage.balances.insert(receiver, storage.balances.get(receiver).try_read().unwrap_or(0) + amount);
    }

    #[storage(read, write)]
    fn send(receiver: Address, amount: u64) {
        let sender = msg_sender().unwrap();
        let sender = match sender {
            Identity::Address(addr) => addr,
            _ => revert(0),
        };

        // Reduce the balance of sender
        let sender_amount = storage.balances.get(sender).try_read().unwrap_or(0);
        assert(sender_amount > amount);
        storage.balances.insert(sender, sender_amount - amount);

        // Increase the balance of receiver
        storage.balances.insert(receiver, storage.balances.get(receiver).try_read().unwrap_or(0) + amount);

        log(Sent {
            from: sender,
            to: receiver,
            amount: amount,
        });
    }
}
// ANCHOR_END: body
