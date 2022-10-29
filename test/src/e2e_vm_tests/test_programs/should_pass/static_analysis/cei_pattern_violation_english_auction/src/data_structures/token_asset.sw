library token_asset;

dep traits;

use ::errors::AssetError;
use traits::Asset;

pub struct TokenAsset {
    /// The amount of the native asset that the struct is representing.
    amount: u64,
    /// The id of the native asset that the struct is representing.
    asset_id: ContractId,
}

impl TokenAsset {
    fn new(amount: u64, asset_id: ContractId) -> Self {
        TokenAsset {
            amount,
            asset_id,
        }
    }
}

impl Asset for TokenAsset {
    fn amount(self) -> u64 {
        self.amount
    }

    fn asset_id(self) -> ContractId {
        self.asset_id
    }
}

impl core::ops::Add for TokenAsset {
    fn add(self, other: Self) -> Self {
        require(self.asset_id() == other.asset_id(), AssetError::AssetsAreNotTheSame);
        ~TokenAsset::new(self.amount() + other.amount(), self.asset_id())
    }
}

impl core::ops::Eq for TokenAsset {
    fn eq(self, other: Self) -> bool {
        self.asset_id() == other.asset_id()
    }
}

impl core::ops::Ord for TokenAsset {
    fn gt(self, other: Self) -> bool {
        require(self.asset_id() == other.asset_id(), AssetError::AssetsAreNotTheSame);
        self.amount() > other.amount()
    }
    fn lt(self, other: Self) -> bool {
        require(self.asset_id() == other.asset_id(), AssetError::AssetsAreNotTheSame);
        self.amount() < other.amount()
    }
}
