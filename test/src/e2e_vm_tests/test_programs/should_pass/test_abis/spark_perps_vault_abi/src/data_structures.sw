library;

pub struct CollateralConfiguration{
    deposit_cap: u64,
    collateral_ratio: u64,
    collateral_scale: u64,
    discount_ratio: u64,
    price_feed: b256,
}