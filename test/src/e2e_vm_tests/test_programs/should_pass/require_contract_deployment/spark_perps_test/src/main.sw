script;
use spark_perps_proxy_abi::{ProxyContract, data_structures::SparkContracts};
use spark_perps_vault_abi::VaultContract;

#[cfg(experimental_new_encoding = false)]
const PROXY_ID = 0x801a92a445fd12138a0caef1c8e9da10b895c968e534b48645fffc87f6faab60;
#[cfg(experimental_new_encoding = false)]
const VAULT_ID = 0x2a139b4822751ff9bbea3c7afafb6f0875fa77165127e93e3faaffd2b5cc19d4;
#[cfg(experimental_new_encoding = false)]
const ACCOUNT_BALANCE_ID = 0xe1e979d6da1027eb262e2f25b851af2fd1b9fccb353c9bcdd603398c49574e15;
#[cfg(experimental_new_encoding = true)]
const PROXY_ID = 0x86c732f6d0d7b427f067b21f81beed93092a3272e33da2e02ea12d0951561d9e;
#[cfg(experimental_new_encoding = true)]
const VAULT_ID = 0x089ad9c09f74d319c4dd17ce8ac1ee2d41d7bb348831eb0b7438f09f82da4205;
#[cfg(experimental_new_encoding = true)]
const ACCOUNT_BALANCE_ID = 0x089ad9c09f74d319c4dd17ce8ac1ee2d41d7bb348831eb0b7438f09f82da4206;

fn main() -> u64 {
    let proxy = abi(ProxyContract, PROXY_ID);
    let vault = abi(VaultContract, VAULT_ID);

    proxy.publish_new_version(Address::from(ACCOUNT_BALANCE_ID), Address::from(VAULT_ID));

    let res = vault.get_free_collateral_by_token();

    res
}
