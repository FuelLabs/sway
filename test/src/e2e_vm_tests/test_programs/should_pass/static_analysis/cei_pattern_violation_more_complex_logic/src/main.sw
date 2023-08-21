contract;

use std::hash::*;

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
    call_frames::{
        contract_id,
        msg_asset_id,
    },
    context::msg_amount,
};

struct DepositKey {
    sender: Identity,
    auction_id: u64,
}

impl Hash for DepositKey {
    fn hash(self, ref mut state: Hasher) {
        self.sender.hash(state);
        self.auction_id.hash(state);
    }
}

storage {
    auctions: StorageMap<u64, Option<u64>> = StorageMap::<u64, Option<u64>> {},
    deposits: StorageMap<DepositKey, Option<AuctionAsset>> = StorageMap::<DepositKey, Option<AuctionAsset>> {},
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
    fn bid(auction_id: u64, _bid_asset: AuctionAsset) {
        let auction = storage.auctions.get(auction_id).try_read();
        require(auction.is_some(), 42);

        let mut _auction = auction.unwrap();
        let sender = msg_sender().unwrap();

        let sender_deposit = storage.deposits.get(DepositKey {sender, auction_id}).try_read();
        let total_bid: AuctionAsset = match sender_deposit {
            Some(_) => {
                AuctionAsset::TokenAsset(42)
            },
            None => {
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

        storage.deposits.insert(DepositKey {sender, auction_id}, Some(AuctionAsset::TokenAsset(42)));
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

        let total_auctions = storage.total_auctions.read();
        storage.deposits.insert(DepositKey {sender:seller, auction_id: total_auctions}, Some(sell_asset));
        storage.auctions.insert(total_auctions, Some(auction));

        storage.total_auctions.write(storage.total_auctions.read() + 1);
        total_auctions
    }

}
