library;
pub mod data_structures;

use data_structures::*;
use std::bytes::Bytes;

abi VaultContract {
    /// Retrieves the collateral balance for a given address.
    ///
    /// # Arguments
    ///
    /// * `address`: [Address] - The address to query for collateral balance.
    ///
    /// # Returns
    ///
    /// * [u64] - The collateral balance of the specified address.
    fn get_collateral_balance();

    /// Retrieves the free collateral for a given trader in token.
    ///
    /// # Arguments
    ///
    /// * `trader`: [Address] - The trader's address whose free collateral is being queried.
    /// * `token`: [AssetId] - The token asset id.
    ///
    /// # Returns
    ///
    /// * I64` - The amount of free collateral available to the trader in giben token.
    fn get_free_collateral_by_token() -> u64;
}
