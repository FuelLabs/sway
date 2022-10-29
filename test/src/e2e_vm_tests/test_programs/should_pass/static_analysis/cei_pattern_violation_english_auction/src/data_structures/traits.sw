library traits;

pub trait Asset {
    fn amount(self) -> u64;
    fn asset_id(self) -> ContractId;
}
