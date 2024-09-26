contract;

use std::asset::transfer;

abi TxOutputChangeContract {
    fn send_assets(to: Address, asset: AssetId, amount: u64);
}

impl TxOutputChangeContract for Contract {
    fn send_assets(to: Address, asset: AssetId, amount: u64) {
        transfer(Identity::Address(to), asset, amount);
    }
}
