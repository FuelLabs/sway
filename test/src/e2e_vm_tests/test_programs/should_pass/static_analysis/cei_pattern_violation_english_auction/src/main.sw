contract;

pub enum AuctionAsset {
    NFTAsset: NFTAsset,
    TokenAsset: TokenAsset,
}

abi EnglishAuction {
    #[storage(read, write)]
    fn bid(auction_id: u64, bid_asset: AuctionAsset);

    #[storage(read, write)]
    fn create(bid_asset: AuctionAsset, duration: u64, inital_price: u64, reserve_price: Option<u64>, seller: Identity, sell_asset: u64) -> AuctionAsset;
}

use std::{
    block::height,
    chain::auth::msg_sender,
    context::{
        call_frames::{
            contract_id,
            msg_asset_id,
        },
        msg_amount,
    },
    storage::StorageMap,
};

storage {
    auctions: StorageMap<u64, Option<u64>> = StorageMap {},
    deposits: StorageMap<(Identity, u64), Option<u64>> = StorageMap {},
}

impl EnglishAuction for Contract {
    #[storage(read, write)]
    fn bid(auction_id: u64, bid_asset: AuctionAsset) {
        let auction = storage.auctions.get(auction_id);
        require(auction.is_some());

        let mut auction = auction.unwrap();
        let sender = msg_sender().unwrap();
        require(sender != auction.seller, UserError::BidderIsSeller);
        require(auction.state == State::Open && height() <= auction.end_block, AccessError::AuctionIsNotOpen);
        require(bid_asset == auction.bid_asset, InputError::IncorrectAssetProvided);

        let sender_deposit = storage.deposits.get((sender, auction_id));
        let total_bid = match sender_deposit {
            Option::Some(AuctionAsset) => {
                bid_asset + sender_deposit.unwrap()
            },
            Option::None(AuctionAsset) => {
                bid_asset
            }
        };

        match total_bid {
            AuctionAsset::NFTAsset(nft_asset) => {
                transfer_nft(nft_asset, sender, Identity::ContractId(contract_id()));
            },
            AuctionAsset::TokenAsset(token_asset) => {
                require(total_bid == 42);
            }
        }

        storage.deposits.insert((sender, auction_id), Option::Some(auction.bid_asset));
    }

    #[storage(read, write)]
    fn create(
        bid_asset: AuctionAsset,
        duration: u64,
        initial_price: u64,
        reserve_price: Option<u64>,
        seller: Identity,
        sell_asset: AuctionAsset,
    ) -> u64 {
        require(reserve_price.is_none() || (reserve_price.is_some() && reserve_price.unwrap() >= initial_price), InitError::ReserveLessThanInitialPrice);
        require(duration != 0, InitError::AuctionDurationNotProvided);

        match bid_asset {
            AuctionAsset::TokenAsset(asset) => {
                require(asset.amount() == 0, InitError::BidAssetAmountNotZero);
            },
            AuctionAsset::NFTAsset(asset) => {
                require(asset.token_id() == 0, InitError::BidAssetAmountNotZero);
            }
        }

        match sell_asset {
            AuctionAsset::TokenAsset(asset) => {
                // Selling tokens
                // TODO: Move this outside the match statement when StorageVec in structs is supported
                require(initial_price != 0, InitError::InitialPriceCannotBeZero);
                require(msg_amount() == asset.amount(), InputError::IncorrectAmountProvided);
                require(msg_asset_id() == asset.asset_id(), InputError::IncorrectAssetProvided);
            },
            AuctionAsset::NFTAsset(asset) => {
                // Selling NFTs
                let sender = msg_sender().unwrap();
                // TODO: Remove this when StorageVec in structs is supported
                require(initial_price == 1, InitError::CannotAcceptMoreThanOneNFT);
                transfer_nft(asset, sender, Identity::ContractId(contract_id()));
            }
        }

        let auction = 42;

        let total_auctions = storage.total_auctions;
        storage.deposits.insert((seller, total_auctions), Option::Some(sell_asset));
        storage.auctions.insert(total_auctions, Option::Some(auction));

        storage.total_auctions += 1;
        total_auctions
    }

}
