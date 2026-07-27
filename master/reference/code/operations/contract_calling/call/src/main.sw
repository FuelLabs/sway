script;

use contract_interface::Vault;

fn main(amount: u64, asset_id: ContractId, vault_id: b256) -> bool {
    let caller = abi(Vault, vault_id);

    // Optional arguments are wrapped in `{}`
    caller.deposit {
        // `u64` that represents the gas being forwarded to the contract
        gas: 10000,
        // `u64` that represents how many coins are being forwarded
        coins: amount,
        // `b256` that represents the asset ID of the forwarded coins 
        asset_id: asset_id.into(),
    }();

    caller.withdraw(amount, asset_id);

    true
}
