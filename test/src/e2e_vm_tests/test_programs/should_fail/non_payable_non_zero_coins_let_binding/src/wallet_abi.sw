library wallet_abi;

abi Wallet {
    #[payable]
    #[storage(read, write)]
    fn receive_funds();

    // non-payable method
    #[storage(read, write)]
    fn send_funds(amount_to_send: u64, recipient_address: Address);
}
