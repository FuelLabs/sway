library auction_asset;

dep nft_asset;
dep token_asset;
dep traits;

use nft_asset::NFTAsset;
use token_asset::TokenAsset;
use traits::Asset;

pub enum AuctionAsset {
    NFTAsset: NFTAsset,
    TokenAsset: TokenAsset,
}

impl Asset for AuctionAsset {
    fn amount(self) -> u64 {
        match self {
            AuctionAsset::NFTAsset(nft_asset) => {
                nft_asset.amount()
            },
            AuctionAsset::TokenAsset(token_asset) => {
                token_asset.amount()
            },
        }
    }

    fn asset_id(self) -> ContractId {
        match self {
            AuctionAsset::NFTAsset(nft_asset) => {
                nft_asset.asset_id()
            },
            AuctionAsset::TokenAsset(token_asset) => {
                token_asset.asset_id()
            },
        }
    }
}

// Formatting error here as described by: https://github.com/FuelLabs/sway/issues/3131
impl core::ops::Add for AuctionAsset {
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (

                AuctionAsset::NFTAsset(nft_asset1),
                AuctionAsset::NFTAsset(nft_asset2),
            ) => {
                AuctionAsset::NFTAsset(nft_asset1 + nft_asset2)
            }
            (

                AuctionAsset::TokenAsset(token_asset1),
                AuctionAsset::TokenAsset(token_asset2),
            ) => {
                AuctionAsset::TokenAsset(token_asset1 + token_asset2)
            },
            _ => {
                revert(0);
            },
        }
    }
}

// Formatting error here as described by: https://github.com/FuelLabs/sway/issues/3131
impl core::ops::Eq for AuctionAsset {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (

                AuctionAsset::NFTAsset(nft_asset1),
                AuctionAsset::NFTAsset(nft_asset2),
            ) => {
                nft_asset1 == nft_asset2
            },
            (

                AuctionAsset::TokenAsset(token_asset1),
                AuctionAsset::TokenAsset(token_asset2),
            ) => {
                token_asset1 == token_asset2
            },
            _ => {
                false
            },
        }
    }
}

// Formatting error here as described by: https://github.com/FuelLabs/sway/issues/3131
impl core::ops::Ord for AuctionAsset {
    fn gt(self, other: Self) -> bool {
        match (self, other) {
            (

                AuctionAsset::NFTAsset(nft_asset1),
                AuctionAsset::NFTAsset(nft_asset2),
            ) => {
                nft_asset1 > nft_asset2
            },
            (

                AuctionAsset::TokenAsset(token_asset1),
                AuctionAsset::TokenAsset(token_asset2),
            ) => {
                token_asset1 > token_asset2
            },
            _ => {
                revert(0);
            },
        }
    }

    fn lt(self, other: Self) -> bool {
        match (self, other) {
            (

                AuctionAsset::NFTAsset(nft_asset1),
                AuctionAsset::NFTAsset(nft_asset2),
            ) => {
                nft_asset1 < nft_asset2
            },
            (

                AuctionAsset::TokenAsset(token_asset1),
                AuctionAsset::TokenAsset(token_asset2),
            ) => {
                token_asset1 < token_asset2
            },
            _ => {
                revert(0);
            },
        }
    }
}
