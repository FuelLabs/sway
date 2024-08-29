contract;

abi AssetIdTestContract {
    fn default_asset_id() -> AssetId;
}

impl AssetIdTestContract for Contract {
    fn default_asset_id() -> AssetId {
        AssetId::default()
    }
}

#[test]
fn asset_id_default() {
    let contract_id = ContractId::from(CONTRACT_ID);
    let asset_id_test_abi = abi(AssetIdTestContract, contract_id.bits());

    let result_asset_id = asset_id_test_abi.default_asset_id();
    let computed_asset_id = AssetId::new(contract_id, SubId::zero());

    assert(result_asset_id == computed_asset_id);
}
