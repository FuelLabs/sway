library;

abi Wallet {
    /// When the BASE_ASSET is sent to this function the internal contract balance is incremented
    #[storage(read, write)]
    fn receive_funds();

    /// Sends `amount_to_send` of the BASE_ASSET to `recipient`
    ///
    /// # Arguments
    ///
    /// - `amount_to_send`: amount of BASE_ASSET to send
    /// - `recipient`: user to send the BASE_ASSET to
    ///
    /// # Reverts
    ///
    /// * When the caller is not the owner of the wallet
    /// * When the amount being sent is greater than the amount in the contract
    #[storage(read, write)]
    fn send_funds(amount_to_send: u64, recipient: Identity);
}
