contract;

use std::storage::storage_vec::*;
use spark_perps_proxy_abi::{ProxyContract, data_structures::SparkContracts};
use std::hash::*;

storage {
    /// A vector of Spark Contracts for different versions.
    spark_contracts: StorageVec<SparkContracts> = StorageVec {},
}

impl ProxyContract for Contract {
    #[storage(read, write)]
    fn publish_new_version(
        account_balance_address: Address,
        vault_address: Address,
    ) {
        storage.spark_contracts.push({
            SparkContracts {
                account_balance_address,
                vault_address,
            }
        });
    }

    #[storage(read)]
    fn get_spark_contracts() -> SparkContracts {
        storage.spark_contracts.get(storage.spark_contracts.len() - 1).unwrap().read()
    }
}
