library;

abi AccountBalanceContract {
    /// Gets the settlement token balance and unrealized profit and loss for a trader.
    ///
    /// # Arguments
    ///
    /// * `trader`: [Address] - The trader whose settlement token balance and PNL are to be retrieved.
    ///
    /// # Returns
    ///
    /// * [(I64, I64)] - settlement token balance and unrealized PNL.
    fn get_settlement_token_balance_and_unrealized_pnl();
}
