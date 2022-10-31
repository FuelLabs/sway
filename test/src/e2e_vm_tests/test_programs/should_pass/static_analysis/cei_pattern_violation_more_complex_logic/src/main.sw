contract;

pub enum AuctionAsset {
    NFTAsset: u64,
    TokenAsset: u64,
}

abi EnglishAuction {
    #[storage(read, write)]
    fn bid(auction_id: u64, bid_asset: AuctionAsset);

    #[storage(read, write)]
    fn create(bid_asset: AuctionAsset, duration: u64, inital_price: u64, reserve_price: Option<u64>, seller: Identity, sell_asset: AuctionAsset) -> u64;
}

abi NFT {
    fn owner_of(token_id: u64) -> Identity;
    fn transfer_from(from: Identity, to: Identity, token_id: u64);
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
    deposits: StorageMap<(Identity, u64), Option<AuctionAsset>> = StorageMap {},
    total_auctions: u64 = 0,
}

const ADDR: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

pub fn transfer_nft(asset: u64, from: Identity, to: Identity) {
    let nft_abi = abi(NFT, ADDR);

    nft_abi.transfer_from(from, to, asset);

    let owner = nft_abi.owner_of(asset);
    require(owner == to, 42);
}

impl EnglishAuction for Contract {
    #[storage(read, write)]
    fn bid(auction_id: u64, bid_asset: AuctionAsset) {
        let auction = storage.auctions.get(auction_id);
        require(auction.is_some(), 42);

        let mut auction = auction.unwrap();
        let sender = msg_sender().unwrap();

        let sender_deposit = storage.deposits.get((sender, auction_id));
        let total_bid: AuctionAsset = match sender_deposit {
            Option::Some(_) => {
                AuctionAsset::TokenAsset(42)
            },
            Option::None => {
                AuctionAsset::NFTAsset(42)
            }
        };

        match total_bid {
            AuctionAsset::NFTAsset(nft_asset) => {
                transfer_nft(nft_asset, sender, Identity::ContractId(contract_id()));
            },
            AuctionAsset::TokenAsset(token_asset) => {
                require(token_asset == 42, 42);
            }
        }

        storage.deposits.insert((sender, auction_id), Option::Some(AuctionAsset::TokenAsset(42)));
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
        require(reserve_price.is_none() || (reserve_price.is_some() && reserve_price.unwrap() >= initial_price), 42);
        require(duration != 0, 42);

        match bid_asset {
            AuctionAsset::TokenAsset(asset) => {
                require(asset == 0, 42);
            },
            AuctionAsset::NFTAsset(asset) => {
                require(asset == 0, 42);
            }
        }

        match sell_asset {
            AuctionAsset::TokenAsset(asset) => {
                require(initial_price != 0, 42);
                require(msg_amount() == asset, 42);
            },
            AuctionAsset::NFTAsset(asset) => {
                // Selling NFTs
                let sender = msg_sender().unwrap();
                // TODO: Remove this when StorageVec in structs is supported
                require(initial_price == 1, 42);
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
