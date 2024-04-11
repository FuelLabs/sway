library;

pub mod data_structures;
use data_structures::SparkContracts;

abi ProxyContract {
    /// Publishes a new version of Spark Contracts with the provided addresses.
    ///
    /// # Arguments
    ///
    /// * `account_balance_address`: [Address] - The address of the account balance contract.
    /// * `clearing_house_address`: [Address] - The address of the clearing house contract.
    /// * `insurance_fund_address`: [Address] - The address of the insurance fund contract.
    /// * `perp_market_address`: [Address] - The address of the perpetual market contract.
    /// * `vault_address`: [Address] - The address of the vault contract.
    /// * `pyth_address`: [Address] - The address of the Pyth oracle contract.
    #[storage(read, write)]
    fn publish_new_version(account_balance_address: Address, vault_address: Address);
    
    /// Retrieves the Spark Contracts for the current version.
    ///
    /// # Returns
    ///
    /// * [SparkContracts] - The Spark Contracts struct for the current version.
    #[storage(read)]
    fn get_spark_contracts() -> SparkContracts;
}
