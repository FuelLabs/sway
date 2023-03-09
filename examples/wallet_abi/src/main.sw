// ANCHOR: abi_library
library;

// ANCHOR: abi
abi Wallet {
    // ANCHOR: receive_funds
    #[storage(read, write), payable]
    fn receive_funds();
    // ANCHOR_END: receive_funds

    // ANCHOR: send_funds
    #[storage(read, write)]
    fn send_funds(amount_to_send: u64, recipient_address: Address);
    // ANCHOR_END: send_funds
}
// ANCHOR: abi
// ANCHOR_END: abi_library
