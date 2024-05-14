contract;

abi AssetIdTestContract {
    fn get_base_asset_id() -> AssetId;
}

impl AssetIdTestContract for Contract {
    fn get_base_asset_id() -> AssetId {
        AssetId::base()
    }
}
