contract;

use spark_perps_vault_abi::VaultContract;
use spark_perps_account_balance_abi::AccountBalanceContract;
use spark_perps_proxy_abi::ProxyContract;

#[cfg(experimental_new_encoding = false)]
const PROXY_ADDRESS = 0x801a92a445fd12138a0caef1c8e9da10b895c968e534b48645fffc87f6faab60;
#[cfg(experimental_new_encoding = true)]
const PROXY_ADDRESS = 0x86c732f6d0d7b427f067b21f81beed93092a3272e33da2e02ea12d0951561d9e;

impl AccountBalanceContract for Contract {
    fn get_settlement_token_balance_and_unrealized_pnl() {
	let proxy_contract = abi(ProxyContract, PROXY_ADDRESS);
	let spark_contracts = proxy_contract.get_spark_contracts();
	let vault_contract = abi(VaultContract, spark_contracts.vault_address.into());
	
	vault_contract.get_collateral_balance();
    }
}
